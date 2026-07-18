use crate::{cpu::memory::Memory, trap::Trap};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl VirtqDesc {
    pub fn read(mem: &Memory, addr: u64) -> Result<Self, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;
        let off = addr - RAM_BASE;
        Ok(Self {
            addr: mem.read64(off)?,
            len: mem.read32(off + 8)?,
            flags: mem.read16(off + 12)?,
            next: mem.read16(off + 14)?,
        })
    }
}
