use crate::{cpu::memory::Memory, trap::Trap};

use super::descriptor::VirtqDesc;

#[derive(Clone, Debug)]
pub struct VirtQueue {
    pub size: u16,
    pub ready: bool,

    pub desc_table: u64,
    pub avail_ring: u64,
    pub used_ring: u64,

    pub last_avail_idx: u16,
}

impl Default for VirtQueue {
    fn default() -> Self {
        Self {
            size: 128,
            ready: false,
            desc_table: 0,
            avail_ring: 0,
            used_ring: 0,
            last_avail_idx: 0,
        }
    }
}

impl VirtQueue {
    pub fn pop_descriptor(&mut self, mem: &Memory) -> Result<Option<(u16, VirtqDesc)>, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;

        if !self.ready {
            return Ok(None);
        }

        let avail_idx = mem.read16(self.avail_ring - RAM_BASE + 2)?;

        if avail_idx == self.last_avail_idx {
            return Ok(None);
        }

        let ring_addr = self.avail_ring + 4 + ((self.last_avail_idx as u64 % self.size as u64) * 2);
        let desc_head = mem.read16(ring_addr - RAM_BASE)?;

        self.last_avail_idx = self.last_avail_idx.wrapping_add(1);

        let desc_addr = self.desc_table + (desc_head as u64 * 16);
        let desc = VirtqDesc::read(mem, desc_addr)?;
        Ok(Some((desc_head, desc)))
    }
}
