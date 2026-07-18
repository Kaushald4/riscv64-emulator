use crate::{
    devices::{
        virtio::{
            descriptor::VirtqDesc,
            device::VirtIODevice,
            device::VirtioContext,
            features::*,
        },
    },
    trap::Trap,
};

use super::backend::BlockBackend;
use super::config::VirtIOBlkConfig;
use super::request::{RequestType, VirtIOBlkReqHeader, VIRTIO_BLK_S_OK, VIRTIO_BLK_S_UNSUPP};

pub const VIRTIO_DEVICE_BLOCK: u32 = 2;
pub const VIRTIO_BLK_F_RO: u64 = 1 << 5;
pub const VIRTIO_BLK_F_FLUSH: u64 = 1 << 9;
pub const VIRTIO_BLK_F_CONFIG_WCE: u64 = 1 << 11;

pub const BLOCK_HOST_FEATURES: u64 = VIRTIO_F_VERSION_1 | VIRTIO_BLK_F_FLUSH | VIRTIO_RING_F_INDIRECT_DESC;

pub struct VirtIOBlock<B: BlockBackend> {
    pub config: VirtIOBlkConfig,
    pub backend: B,
}

impl<B: BlockBackend> VirtIOBlock<B> {
    pub fn new(backend: B) -> Self {
        let capacity = backend.sector_count();

        Self {
            config: VirtIOBlkConfig { capacity, blk_size: 512, ..Default::default() },
            backend,
        }
    }
}

impl<B: BlockBackend> VirtIODevice for VirtIOBlock<B> {
    fn device_id(&self) -> u32 {
        VIRTIO_DEVICE_BLOCK
    }

    fn host_features(&self) -> u64 {
        BLOCK_HOST_FEATURES
    }

    fn read_config32(&self, offset: u64) -> u32 {
        match offset {
            0x00 => self.config.capacity as u32,
            0x04 => (self.config.capacity >> 32) as u32,
            0x08 => self.config.size_max,
            0x0c => self.config.seg_max,
            0x14 => self.config.blk_size,
            _ => 0,
        }
    }

    fn process_descriptor_chain(&mut self, ctx: &mut VirtioContext, chain: &[VirtqDesc], _queue_idx: u16) -> Result<u32, Trap> {
        let header_desc = &chain[0];
        let header = VirtIOBlkReqHeader::read(ctx.memory, header_desc.addr)?;
        let req_type = RequestType::from(header.request_type);

        let status_desc = chain.last().unwrap();

        Ok(match req_type {
            RequestType::In => {
                let mut current_sector_offset = header.sector as usize * 512;
                let mut total_data = 0u32;

                for data_desc in &chain[1..chain.len() - 1] {
                    let length = data_desc.len as usize;
                    let chunk = self.backend.read(current_sector_offset, length);
                    ctx.memory.load(data_desc.addr - 0x8000_0000, &chunk)?;
                    current_sector_offset += length;
                    total_data += data_desc.len;
                }
                ctx.memory.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                total_data + 1
            }

            RequestType::Out => {
                let mut current_sector_offset = header.sector as usize * 512;

                for data_desc in &chain[1..chain.len() - 1] {
                    let length = data_desc.len as usize;
                    let mut buf = vec![0u8; length];
                    ctx.memory.read_bulk(data_desc.addr - 0x8000_0000, &mut buf)?;
                    self.backend.write(current_sector_offset, &buf);
                    current_sector_offset += length;
                }
                ctx.memory.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                1
            }

            RequestType::Flush => {
                ctx.memory.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                1
            }

            RequestType::GetId => {
                let data_desc = &chain[1];
                let mut buf = [0u8; 20];
                let id = b"glasshart-block";
                let len = id.len().min(buf.len());
                buf[..len].copy_from_slice(&id[..len]);
                ctx.memory.load(data_desc.addr - 0x8000_0000, &buf)?;
                ctx.memory.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                data_desc.len + 1
            }

            RequestType::Unknown => {
                ctx.memory.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_UNSUPP)?;
                1
            }
        })
    }
}
