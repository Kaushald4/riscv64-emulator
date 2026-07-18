use crate::{
    devices::virtio::{
        descriptor::VirtqDesc,
        device::VirtIODevice,
        device::VirtioContext,
        features::*,
    },
    trap::Trap,
};

use super::backend::NetworkBackend;
use super::config::VirtIONetConfig;
use super::header::VirtioNetHdr;

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
    | VIRTIO_NET_F_CSUM
    | VIRTIO_NET_F_GUEST_CSUM
    | VIRTIO_NET_F_MAC
    | VIRTIO_NET_F_GUEST_TSO4
    | VIRTIO_NET_F_GUEST_TSO6
    | VIRTIO_NET_F_HOST_TSO4
    | VIRTIO_NET_F_HOST_TSO6
    | VIRTIO_NET_F_STATUS
    | VIRTIO_RING_F_INDIRECT_DESC;

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
            // RX queue: device → driver (receive packet from host)
            0 => {
                Ok(0)
            }
            // TX queue: driver → device (transmit packet to host)
            _ => {
                let header_desc = &chain[0];
                let _hdr = VirtioNetHdr::read(ctx.memory, header_desc.addr)?;

                let mut packet = Vec::new();
                for data_desc in &chain[1..] {
                    let mut buf = vec![0u8; data_desc.len as usize];
                    ctx.memory.read_bulk(data_desc.addr - 0x8000_0000, &mut buf)?;
                    packet.extend_from_slice(&buf);
                }

                self.backend.send(&packet).ok();
                Ok(header_desc.len)
            }
        }
    }
}
