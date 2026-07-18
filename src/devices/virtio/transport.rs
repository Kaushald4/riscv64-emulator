use crate::{cpu::memory::Memory, trap::Trap};

use super::descriptor::VirtqDesc;
use super::queue::VirtQueue;

const RAM_BASE: u64 = 0x8000_0000;

pub fn write_used_ring(mem: &mut Memory, queue: &VirtQueue, desc_head: u16, len: u32) -> Result<(), Trap> {
    let used_ring = queue.used_ring;
    let idx = mem.read16(used_ring - RAM_BASE + 2)?;

    let elem_addr = used_ring + 4 + (idx as u64 % queue.size as u64) * 8;
    mem.write32(elem_addr - RAM_BASE, desc_head as u32)?;
    mem.write32(elem_addr + 4 - RAM_BASE, len)?;

    mem.write16(used_ring - RAM_BASE + 2, idx.wrapping_add(1))?;
    Ok(())
}

pub fn collect_chain(mem: &Memory, queue: &VirtQueue, head_desc: VirtqDesc) -> Result<Vec<VirtqDesc>, Trap> {
    let mut chain = Vec::with_capacity(8);

    // VIRTQ_DESC_F_INDIRECT (0x04)
    if head_desc.flags & 0x04 != 0 {
        let indirect_table = head_desc.addr;
        let mut next_idx = 0u16;
        loop {
            let d = VirtqDesc::read(mem, indirect_table + (next_idx as u64 * 16))?;
            let has_next = (d.flags & 0x01) != 0;
            chain.push(d);
            if !has_next {
                break;
            }
            next_idx = d.next;
        }
        return Ok(chain);
    }

    let mut current = head_desc;
    let mut visited: u16 = 0;

    loop {
        chain.push(current);

        if current.flags & 1 == 0 {
            break;
        }

        visited += 1;
        if visited >= queue.size {
            break;
        }

        let next_addr = queue.desc_table + (current.next as u64 * 16);
        current = VirtqDesc::read(mem, next_addr)?;
    }

    Ok(chain)
}
