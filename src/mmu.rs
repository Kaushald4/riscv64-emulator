use crate::{
    cpu::Cpu,
    mmu::{access_type::AccessType, walker::PageWalker},
    trap::Trap,
};

pub mod access_type;
pub mod address;
pub mod pte;
pub mod satp;
pub mod translation;
pub mod walker;

pub struct Mmu;

impl Mmu {
    #[inline]
    pub fn translate(cpu: &mut Cpu, virtual_address: u64, access: AccessType) -> Result<u64, Trap> {
        Ok(PageWalker::translate(cpu, virtual_address, access)?.physical_address)
    }

    pub fn read8(cpu: &mut Cpu, addr: u64) -> Result<u8, Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
        cpu.bus.read8(pa)
    }

    pub fn read16(cpu: &mut Cpu, addr: u64) -> Result<u16, Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
        cpu.bus.read16(pa)
    }

    pub fn read32(cpu: &mut Cpu, addr: u64) -> Result<u32, Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
        cpu.bus.read32(pa)
    }

    pub fn read64(cpu: &mut Cpu, addr: u64) -> Result<u64, Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
        cpu.bus.read64(pa)
    }

    pub fn write8(cpu: &mut Cpu, addr: u64, value: u8) -> Result<(), Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
        cpu.bus.write8(pa, value)
    }

    pub fn write16(cpu: &mut Cpu, addr: u64, value: u16) -> Result<(), Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
        cpu.bus.write16(pa, value)
    }

    pub fn write32(cpu: &mut Cpu, addr: u64, value: u32) -> Result<(), Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
        cpu.bus.write32(pa, value)
    }

    pub fn write64(cpu: &mut Cpu, addr: u64, value: u64) -> Result<(), Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
        cpu.bus.write64(pa, value)
    }
}
