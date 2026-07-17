use crate::{cpu::Cpu, trap::Trap};

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
    pub fn pop_descriptor(&mut self, cpu: &mut Cpu) -> Result<Option<VirtqDesc>, Trap> {
        use crate::mmu::Mmu;

        if !self.ready {
            return Ok(None);
        }

        let avail_idx = Mmu::read16(cpu, self.avail_ring + 2)?;

        if avail_idx == self.last_avail_idx {
            return Ok(None);
        }

        let ring_addr = self.avail_ring + 4 + ((self.last_avail_idx as u64 % self.size as u64) * 2);

        let desc_index = Mmu::read16(cpu, ring_addr)?;

        self.last_avail_idx = self.last_avail_idx.wrapping_add(1);

        let desc_addr = self.desc_table + (desc_index as u64 * 16);

        Ok(Some(VirtqDesc::read(cpu, desc_addr)?))
    }
}
