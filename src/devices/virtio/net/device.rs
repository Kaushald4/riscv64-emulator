use crate::{
    cpu::memory::Memory,
    devices::virtio::{
        descriptor::VirtqDesc,
        device::{VirtIODevice, VirtioContext},
        features::*,
        queue::VirtQueue,
        transport,
    },
    trap::Trap,
};

use super::backend::NetworkBackend;
use super::config::VirtIONetConfig;
use super::header::{VirtioNetHdr, VIRTIO_NET_HDR_SIZE};

pub const VIRTIO_DEVICE_NET: u32 = 1;

pub const VIRTIO_NET_F_CSUM: u64 = 1 << 0;
pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1 << 1;
pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
pub const VIRTIO_NET_F_GUEST_TSO4: u64 = 1 << 7;
pub const VIRTIO_NET_F_GUEST_TSO6: u64 = 1 << 8;
pub const VIRTIO_NET_F_HOST_TSO4: u64 = 1 << 11;
pub const VIRTIO_NET_F_HOST_TSO6: u64 = 1 << 12;
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;

pub const NET_HOST_FEATURES: u64 =
    VIRTIO_F_VERSION_1
    | VIRTIO_NET_F_GUEST_CSUM
    | VIRTIO_NET_F_MAC
    | VIRTIO_NET_F_STATUS;

/// largest Ethernet frame we accept from the backend (Ethernet MTU + 14 byte header).
pub const MAX_FRAME_SIZE: usize = 1514;

const RAM_BASE: u64 = 0x8000_0000;

pub struct VirtIONet {
    pub backend: Box<dyn NetworkBackend>,
    pub config: VirtIONetConfig,
}

impl VirtIONet {
    pub fn new(backend: Box<dyn NetworkBackend>) -> Self {
        let mac = backend.mac_address();
        Self {
            backend,
            config: VirtIONetConfig { mac, status: 1, max_virtqueue_pairs: 1, mtu: 1500 },
        }
    }

    /// Drain the receive (RX) queue.
    ///
    /// For every frame the backend has queued, pop one driver-provided RX
    /// buffer off the avail ring, write the virtio-net header followed by
    /// the frame bytes, post it to the used ring, and signal an interrupt.
    ///
    /// Returns `true` if at least one buffer was filled (so the caller should
    /// raise the interrupt line).
    pub fn drain_rx(&mut self, ctx: &mut VirtioContext, queue: &mut VirtQueue) -> Result<bool, Trap> {
        if !queue.ready {
            return Ok(false);
        }

        let mut triggered = false;
        let mut frame = [0u8; MAX_FRAME_SIZE];

        loop {
            // Pop the next RX buffer the driver posted BEFORE polling the
            // backend. The WebRTC backend's `receive()` is destructive —
            // it dequeues from the JS-side rxQueue — so calling it without
            // a destination buffer would silently destroy the frame.
            // Popping first and "unpopping" if the backend has nothing is
            // the only correct order.
            let Some((desc_head, head_desc)) = queue.pop_descriptor(ctx.memory)? else {
                // no buffers posted — leave the backend alone.
                break;
            };

            // try to pull a frame from the backend.
            let n = match self.backend.receive(&mut frame) {
                Ok(Some(n)) => n,
                Ok(None) | Err(_) => {
                    // backend had nothing. Put the buffer back so the
                    // next poll can use it for a real frame.
                    queue.unpop();
                    break;
                }
            };

            let chain = transport::collect_chain(ctx.memory, queue, head_desc)?;
            let written = Self::write_rx_frame(ctx.memory, &chain, &frame[..n])?;
            transport::write_used_ring(ctx.memory, queue, desc_head, written)?;
            triggered = true;
        }

        Ok(triggered)
    }

    /// Write a received frame into a driver-provided descriptor chain.
    ///
    /// Without `VIRTIO_NET_F_MRG_RXBUF` (which we don't advertise), the
    /// driver hands us one or more writable descriptors. The first must hold
    /// the 12-byte virtio-net header; the frame payload goes immediately
    /// after the header in the same descriptor, then spills into the rest of
    /// the chain.
    
    fn write_rx_frame(mem: &mut Memory, chain: &[VirtqDesc], frame: &[u8]) -> Result<u32, Trap> {
    let mut hdr = VirtioNetHdr::default();
    
    // tell the guest kernel - the hypervisor has already validated this checksum. DO NOT DROP IT.
    hdr.flags = 2;
    hdr.write(mem, chain[0].addr)?;

    let mut written = VIRTIO_NET_HDR_SIZE as u32;
    let mut remaining: &[u8] = frame;

    if chain[0].len as usize > VIRTIO_NET_HDR_SIZE {
        let room = (chain[0].len as usize) - VIRTIO_NET_HDR_SIZE;
        let take = remaining.len().min(room);
        mem.load(chain[0].addr - RAM_BASE + VIRTIO_NET_HDR_SIZE as u64, &remaining[..take])?;
        remaining = &remaining[take..];
        written += take as u32;
    }

    for data_desc in &chain[1..] {
        if remaining.is_empty() {
            break;
        }
        let room = data_desc.len as usize;
        let take = remaining.len().min(room);
        mem.load(data_desc.addr - RAM_BASE, &remaining[..take])?;
        remaining = &remaining[take..];
        written += take as u32;
    }

    Ok(written)
}


}

impl VirtIODevice for VirtIONet {
    fn device_id(&self) -> u32 {
        VIRTIO_DEVICE_NET
    }

    fn host_features(&self) -> u64 {
        NET_HOST_FEATURES
    }

    fn read_config32(&self, offset: u64) -> u32 {
        self.config.read32(offset)
    }




    fn process_descriptor_chain(&mut self, ctx: &mut VirtioContext, chain: &[VirtqDesc], queue_idx: u16) -> Result<u32, Trap>  {
    match queue_idx {
        0 => Ok(0),
        _ => {
            // 1. flatten the ENTIRE scatter-gather chain into a single contiguous buffer first.
            // this guarantees we never chop a packet, regardless of how Linux fragments it.
            let mut raw_buffer = Vec::new();
            let mut total_len = 0;
            
            for desc in chain {
                let mut buf = vec![0u8; desc.len as usize];
                ctx.memory.read_bulk(desc.addr - RAM_BASE, &mut buf)?;
                raw_buffer.extend_from_slice(&buf);
                total_len += desc.len as u32;
            }

            if raw_buffer.len() < VIRTIO_NET_HDR_SIZE {
                return Ok(total_len); 
            }

            let flags = raw_buffer[0];
            let csum_start = u16::from_le_bytes([raw_buffer[6], raw_buffer[7]]) as usize;
            let csum_offset = u16::from_le_bytes([raw_buffer[8], raw_buffer[9]]) as usize;

            // 2. extract the actual Ethernet frame
            let mut packet = raw_buffer[VIRTIO_NET_HDR_SIZE..].to_vec();

            // 3. Fix IPv4 Ethernet Padding
            if packet.len() >= 14 && packet[12] == 0x08 && packet[13] == 0x00 {
                let ip_total_len = u16::from_be_bytes([packet[16], packet[17]]) as usize;
                let expected_len = 14 + ip_total_len;
                if packet.len() > expected_len {
                    packet.truncate(expected_len);
                }
            }

            // 4. hardware checksum offload
            if (flags & 1) != 0 { // VIRTIO_NET_HDR_F_NEEDS_CSUM
                if csum_start + csum_offset + 1 < packet.len() {
                    let mut sum: u32 = 0;
                    for chunk in packet[csum_start..].chunks(2) {
                        if chunk.len() == 2 {
                            sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
                        } else {
                            sum += (chunk[0] as u32) << 8;
                        }
                    }
                    while sum > 0xFFFF {
                        sum = (sum & 0xFFFF) + (sum >> 16);
                    }
                    let csum = !(sum as u16);
                    packet[csum_start + csum_offset] = (csum >> 8) as u8;
                    packet[csum_start + csum_offset + 1] = (csum & 0xFF) as u8;
                }
            }

            self.backend.send(&packet).ok();
            
            // ror TX, virtios spec dictates we return the length processed. 
            // patched this in transport.rs earlier to report 0 to the ring.
            Ok(total_len)
        }
    }
}

 

}
