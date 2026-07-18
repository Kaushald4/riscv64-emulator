use crate::{
    cpu::memory::Memory,
    devices::{
        plic::Plic,
        virtio::{device::VirtIODevice, features::*, queue::VirtQueue, transport},
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

    fn process_queue(&mut self, mem: &mut Memory, queue: &mut VirtQueue, plic: &mut Plic, interrupt_status: &mut u32) -> Result<(), Trap> {
        let mut triggered = false;

        loop {
            let Some((desc_head, head_desc)) = queue.pop_descriptor(mem)? else {
                break;
            };

            let chain = transport::collect_chain(mem, queue, head_desc)?;

            let header_desc = &chain[0];
            let header = VirtIOBlkReqHeader::read(mem, header_desc.addr)?;
            let req_type = RequestType::from(header.request_type);

            // Last descriptor in chain is always the status byte.
            let status_desc = chain.last().unwrap();
            let written_len;

            match req_type {
                RequestType::In => {
                    let mut current_sector_offset = header.sector as usize * 512;
                    let mut total_data = 0u32;

                    for data_desc in &chain[1..chain.len() - 1] {
                        let length = data_desc.len as usize;
                        let chunk = self.backend.read(current_sector_offset, length);
                        mem.load(data_desc.addr - 0x8000_0000, &chunk)?;
                        current_sector_offset += length;
                        total_data += data_desc.len;
                    }
                    mem.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                    written_len = total_data + 1;
                }

                RequestType::Out => {
                    let mut current_sector_offset = header.sector as usize * 512;

                    for data_desc in &chain[1..chain.len() - 1] {
                        let length = data_desc.len as usize;
                        let mut buf = vec![0u8; length];
                        mem.read_bulk(data_desc.addr - 0x8000_0000, &mut buf)?;
                        self.backend.write(current_sector_offset, &buf);
                        current_sector_offset += length;
                    }
                    mem.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                    written_len = 1;
                }

                RequestType::Flush => {
                    mem.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                    written_len = 1;
                }

                RequestType::GetId => {
                    let data_desc = &chain[1];
                    let mut buf = [0u8; 20];
                    let id = b"glasshart-block";
                    let len = id.len().min(buf.len());
                    buf[..len].copy_from_slice(&id[..len]);
                    mem.load(data_desc.addr - 0x8000_0000, &buf)?;
                    mem.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_OK)?;
                    written_len = data_desc.len + 1;
                }

                RequestType::Unknown => {
                    mem.write8(status_desc.addr - 0x8000_0000, VIRTIO_BLK_S_UNSUPP)?;
                    written_len = 1;
                }
            }

            transport::write_used_ring(mem, queue, desc_head, written_len)?;
            triggered = true;
        }

        if triggered {
            *interrupt_status |= 0x1;
            plic.trigger_interrupt(1);
        }

        Ok(())
    }
}
