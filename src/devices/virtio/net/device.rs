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
use super::gro::{self, GsoMeta};
use super::header::{VirtioNetHdr, VIRTIO_NET_HDR_F_NEEDS_CSUM, VIRTIO_NET_HDR_F_DATA_VALID, VIRTIO_NET_HDR_SIZE};

pub const VIRTIO_DEVICE_NET: u32 = 1;

pub const VIRTIO_NET_F_CSUM: u64 = 1 << 0;
pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1 << 1;
pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
pub const VIRTIO_NET_F_GUEST_TSO4: u64 = 1 << 7;
pub const VIRTIO_NET_F_GUEST_TSO6: u64 = 1 << 8;
pub const VIRTIO_NET_F_HOST_TSO4: u64 = 1 << 11;
pub const VIRTIO_NET_F_HOST_TSO6: u64 = 1 << 12;
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;

/// features the device advertises to the guest.
///
/// GUEST_TSO4 is ENABLED — this lets the device deliver coalesced GSO
/// segments (multiple TCP frames merged into one large segment with a
/// GSO header). The guest's virtio-net driver splits them back into
/// individual TCP segments.
///
/// This is the throughput fix for emulated networking - without GRO, each
/// 1500-byte frame costs the guest ~22ms of CPU (virtio interrupt + TCP
/// softirq + socket + wget read), limiting throughput to ~64 KB/s. With
/// GRO, 8-32 frames are coalesced into one GSO segment, cutting per-byte
/// CPU by 8-32x. The guest processes one interrupt instead of 32, and
/// the TCP stack handles one large segment instead of 32 small ones.
pub const NET_HOST_FEATURES: u64 =
    VIRTIO_F_VERSION_1
    | VIRTIO_NET_F_GUEST_CSUM
    | VIRTIO_NET_F_GUEST_TSO4
    | VIRTIO_NET_F_MAC
    | VIRTIO_NET_F_STATUS;

/// largest ethernet frame we accept from the backend (Ethernet MTU + 14 byte header).
pub const MAX_FRAME_SIZE: usize = 1514;

/// number of frames to pull from the backend per drain_rx call. This is
/// the GRO batch size — pulling more frames gives the coalescing logic
/// more to work with. 64 is enough to coalesce a full 64KB TCP window.
const GRO_BATCH: usize = 64;

const RAM_BASE: u64 = 0x8000_0000;

pub struct VirtIONet {
    pub backend: Box<dyn NetworkBackend>,
    pub config: VirtIONetConfig,
    /// GRO accumulator - holds frames across drain_rx calls so the
    /// coalescing logic has enough segments to work with. Without this,
    /// each drain_rx sees only 1-2 frames (the WebRTC message rate is
    /// ~50/sec) and can't coalesce anything. With the accumulator,
    /// frames from multiple drain_rx calls are pooled until we have
    /// enough (8+) to coalesce, then delivered as GSO segments.
    gro_buffer: Vec<Vec<u8>>,
    /// diagnostic counters for GRO, exposed via glasshart_net_stats().
    pub gro_frames_in: u64,
    pub gro_segments_out: u64,
}

impl VirtIONet {
    pub fn new(backend: Box<dyn NetworkBackend>) -> Self {
        let mac = backend.mac_address();
        Self {
            backend,
            config: VirtIONetConfig { mac, status: 1, max_virtqueue_pairs: 1, mtu: 1500 },
            gro_buffer: Vec::new(),
            gro_frames_in: 0,
            gro_segments_out: 0,
        }
    }

    /// drain the receive (RX) queue with GRO (Generic Receive Offload).
    ///
    /// pulls up to GRO_BATCH frames from the backend, coalesces
    /// consecutive TCP segments into large GSO segments, then delivers
    /// each segment to the guest's RX buffers. This is the single most
    /// important optimization for network throughput: instead of
    /// delivering 32 separate 1500-byte frames (each triggering a
    /// virtio interrupt + TCP softirq), GRO delivers ONE ~44KB segment,
    /// cutting per-byte CPU overhead by up to 32x.
    ///
    /// if the guest hasn't negotiated VIRTIO_NET_F_GUEST_TSO4, GRO is
    /// skipped and each frame is delivered individually.
    pub fn drain_rx(&mut self, ctx: &mut VirtioContext, queue: &mut VirtQueue) -> Result<bool, Trap> {
        if !queue.ready {
            return Ok(false);
        }

        let gro_enabled = (ctx.driver_features & VIRTIO_NET_F_GUEST_TSO4) != 0;

        if !gro_enabled {
            return self.drain_rx_simple(ctx, queue);
        }

        // pull up to GRO_BATCH frames from the backend and add them
        // to the GRO accumulator.
        let old_len = self.gro_buffer.len();
        let mut frame_buf = [0u8; MAX_FRAME_SIZE];
        for _ in 0..GRO_BATCH {
            let n = match self.backend.receive(&mut frame_buf) {
                Ok(Some(n)) => n,
                Ok(None) | Err(_) => break,
            };
            self.gro_buffer.push(frame_buf[..n].to_vec());
        }
        let added = self.gro_buffer.len() - old_len;

        // deliver frames if:
        //   1. we have enough to fill a large GSO segment (>= 32 frames), OR
        //   2. the buffer is full (>= 64 frames), OR
        //   3. no new frames arrived this call but we have some held
        //      from previous calls — flush them so they don't sit
        //      forever (e.g. end of a burst, or a lone non-TCP frame).
        //
        // 32 frames × 1388 bytes = 44 KB per GSO segment. This gives
        // 17x interrupt reduction and good RTT amortization.
        let should_deliver = self.gro_buffer.len() >= 32
            || self.gro_buffer.len() >= GRO_BATCH
            || (added == 0 && !self.gro_buffer.is_empty());

        if !should_deliver {
            return Ok(false);
        }

        if self.gro_buffer.is_empty() {
            return Ok(false);
        }

        // coalesce TCP segments via GRO.
        let frames = std::mem::take(&mut self.gro_buffer);
        self.gro_frames_in += frames.len() as u64;
        let coalesced = gro::gro_coalesce(&frames);
        self.gro_segments_out += coalesced.len() as u64;

        // deliver each segment to the guest's RX buffers.
        let mut triggered = false;
        for output in coalesced {
            let Some((desc_head, head_desc)) = queue.pop_descriptor(ctx.memory)? else {
                // queue full — put remaining frames back in the buffer
                // for the next drain_rx call.
                // we can't "put back" coalesced frames easily.
                // just drop them — the guest's TCP stack will retransmit.
                break;
            };

            let chain = transport::collect_chain(ctx.memory, queue, head_desc)?;
            let written = Self::write_rx_frame(ctx.memory, &chain, &output.frame, output.gso)?;
            transport::write_used_ring(ctx.memory, queue, desc_head, written)?;
            triggered = true;
        }

        Ok(triggered)
    }

    /// simple RX path without GRO — used when the guest hasn't
    /// negotiated VIRTIO_NET_F_GUEST_TSO4.
    fn drain_rx_simple(&mut self, ctx: &mut VirtioContext, queue: &mut VirtQueue) -> Result<bool, Trap> {
        let mut triggered = false;
        let mut frame = [0u8; MAX_FRAME_SIZE];

        loop {
            let Some((desc_head, head_desc)) = queue.pop_descriptor(ctx.memory)? else {
                break;
            };

            let n = match self.backend.receive(&mut frame) {
                Ok(Some(n)) => n,
                Ok(None) | Err(_) => {
                    queue.unpop();
                    break;
                }
            };

            let chain = transport::collect_chain(ctx.memory, queue, head_desc)?;
            let written = Self::write_rx_frame(ctx.memory, &chain, &frame[..n], None)?;
            transport::write_used_ring(ctx.memory, queue, desc_head, written)?;
            triggered = true;
        }

        Ok(triggered)
    }

    /// write a received frame into a driver-provided descriptor chain.
    ///
    /// if `gso` is `Some`, the frame is a coalesced GSO segment — the
    /// virtio-net header gets:
    ///   flags      = NEEDS_CSUM (guest computes per-split checksum)
    ///   gso_type   = TCPV4
    ///   hdr_len    = actual header length (eth + IP + TCP)
    ///   gso_size   = MSS (first segment's payload length)
    ///   csum_start = offset to TCP header (eth + IP)
    ///   csum_offset= 16 (TCP checksum field within TCP header)
    ///
    /// for non-GSO frames, we use DATA_VALID — the relay's tap-bridge
    /// already verified/fixed the checksum.
    fn write_rx_frame(
        mem: &mut Memory,
        chain: &[VirtqDesc],
        frame: &[u8],
        gso: Option<GsoMeta>,
    ) -> Result<u32, Trap> {
        let mut hdr = VirtioNetHdr::default();

        if let Some(g) = gso {
            hdr.flags = VIRTIO_NET_HDR_F_NEEDS_CSUM;
            hdr.gso_type = 1; // VIRTIO_NET_HDR_GSO_TCPV4
            hdr.hdr_len = g.hdr_len;
            hdr.gso_size = g.mss;
            hdr.csum_start = g.csum_start;
            hdr.csum_offset = 16;
        } else {
            hdr.flags = VIRTIO_NET_HDR_F_DATA_VALID;
        }

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
            if remaining.is_empty() { break; }
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

    fn process_descriptor_chain(&mut self, ctx: &mut VirtioContext, chain: &[VirtqDesc], queue_idx: u16) -> Result<u32, Trap> {
        match queue_idx {
            0 => Ok(0),
            _ => {
                let header_desc = &chain[0];
                let _hdr = VirtioNetHdr::read(ctx.memory, header_desc.addr)?;

                let mut packet = Vec::with_capacity(1514);

                let hdr_len = VIRTIO_NET_HDR_SIZE as u32;
                if header_desc.len > hdr_len {
                    let payload_len = (header_desc.len - hdr_len) as usize;
                    let mut buf = vec![0u8; payload_len];
                    ctx.memory.read_bulk(
                        header_desc.addr - RAM_BASE + hdr_len as u64,
                        &mut buf,
                    )?;
                    packet.extend_from_slice(&buf);
                }

                for data_desc in &chain[1..] {
                    let mut buf = vec![0u8; data_desc.len as usize];
                    ctx.memory.read_bulk(data_desc.addr - RAM_BASE, &mut buf)?;
                    packet.extend_from_slice(&buf);
                }

                self.backend.send(&packet).ok();
                Ok(header_desc.len)
            }
        }
    }
}
