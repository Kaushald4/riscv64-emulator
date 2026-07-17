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
        // check if access fits entirely within the 4KB (0x1000) page
        if (addr & 0xFFF) + 2 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
            cpu.bus.read16(pa)
        } else {
            // crosses page boundary: read byte-by-byte
            let mut value = 0u16;
            for i in 0..2 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Read)?;
                value |= (cpu.bus.read8(pa)? as u16) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn read32(cpu: &mut Cpu, addr: u64) -> Result<u32, Trap> {
        if (addr & 0xFFF) + 4 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
            cpu.bus.read32(pa)
        } else {
            // crosses page boundary: read byte-by-byte
            let mut value = 0u32;
            for i in 0..4 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Read)?;
                value |= (cpu.bus.read8(pa)? as u32) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn read64(cpu: &mut Cpu, addr: u64) -> Result<u64, Trap> {
        if (addr & 0xFFF) + 8 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
            cpu.bus.read64(pa)
        } else {
            // crosses page boundary: read byte-by-byte
            let mut value = 0u64;
            for i in 0..8 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Read)?;
                value |= (cpu.bus.read8(pa)? as u64) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn write8(cpu: &mut Cpu, addr: u64, value: u8) -> Result<(), Trap> {
        let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
        cpu.bus.write8(pa, value)
    }

    pub fn write16(cpu: &mut Cpu, addr: u64, value: u16) -> Result<(), Trap> {
        if (addr & 0xFFF) + 2 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
            cpu.bus.write16(pa, value)
        } else {
            // crosses page boundary: write byte-by-byte
            for i in 0..2 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Write)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }

    pub fn write32(cpu: &mut Cpu, addr: u64, value: u32) -> Result<(), Trap> {
        if (addr & 0xFFF) + 4 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
            cpu.bus.write32(pa, value)
        } else {
            // crosses page boundary: write byte-by-byte
            for i in 0..4 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Write)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }

    pub fn write64(cpu: &mut Cpu, addr: u64, value: u64) -> Result<(), Trap> {
        if (addr & 0xFFF) + 8 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
            cpu.bus.write64(pa, value)
        } else {
            // crosses page boundary: write byte-by-byte
            for i in 0..8 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Write)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }
}
