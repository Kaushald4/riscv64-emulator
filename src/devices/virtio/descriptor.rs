#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}
impl VirtqDesc {
    pub fn read(cpu: &mut crate::cpu::Cpu, addr: u64) -> Result<Self, crate::trap::Trap> {
        use crate::mmu::Mmu;

        Ok(Self {
            addr: Mmu::read64(cpu, addr)?,
            len: Mmu::read32(cpu, addr + 8)?,
            flags: Mmu::read16(cpu, addr + 12)?,
            next: Mmu::read16(cpu, addr + 14)?,
        })
    }
}
