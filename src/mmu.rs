use crate::{
    cpu::Cpu,
    mmu::{access_type::AccessType, walker::PageWalker},
    trap::Trap,
};

pub mod access_type;
pub mod address;
pub mod pte;
pub mod satp;
pub mod tlb;
pub mod translation;
pub mod walker;

pub struct Mmu;

impl Mmu {
    #[inline]
    pub fn translate(cpu: &mut Cpu, virtual_address: u64, access: AccessType) -> Result<u64, Trap> {
        PageWalker::translate(cpu, virtual_address, access)
    }

    /// Cached data translation. Uses a single-entry page cache per access
    /// type (read/write) to skip the full TLB lookup + permission check
    /// for same-page accesses. This eliminates ~14% of runtime that
    /// translate() was costing for data accesses.
    #[inline]
    fn translate_data(cpu: &mut Cpu, addr: u64, is_write: bool) -> Result<u64, Trap> {
        let vpn = addr >> 12;

        if is_write {
            if cpu.data_write_valid && cpu.data_write_vpn == vpn {
                return Ok(cpu.data_write_pa.wrapping_add(addr.wrapping_sub(cpu.data_write_va)));
            }
            let pa = Mmu::translate(cpu, addr, AccessType::Write)?;
            cpu.data_write_vpn = vpn;
            cpu.data_write_pa = pa;
            cpu.data_write_va = addr;
            cpu.data_write_valid = true;
            Ok(pa)
        } else {
            if cpu.data_read_valid && cpu.data_read_vpn == vpn {
                return Ok(cpu.data_read_pa.wrapping_add(addr.wrapping_sub(cpu.data_read_va)));
            }
            let pa = Mmu::translate(cpu, addr, AccessType::Read)?;
            cpu.data_read_vpn = vpn;
            cpu.data_read_pa = pa;
            cpu.data_read_va = addr;
            cpu.data_read_valid = true;
            Ok(pa)
        }
    }

    pub fn read8(cpu: &mut Cpu, addr: u64) -> Result<u8, Trap> {
        let pa = Self::translate_data(cpu, addr, false)?;
        cpu.bus.read8(pa)
    }

    pub fn read16(cpu: &mut Cpu, addr: u64) -> Result<u16, Trap> {
        if (addr & 0xFFF) + 2 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, false)?;
            cpu.bus.read16(pa)
        } else {
            let mut value = 0u16;
            for i in 0..2 {
                let pa = Self::translate_data(cpu, addr + i, false)?;
                value |= (cpu.bus.read8(pa)? as u16) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn read32(cpu: &mut Cpu, addr: u64) -> Result<u32, Trap> {
        if (addr & 0xFFF) + 4 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, false)?;
            cpu.bus.read32(pa)
        } else {
            let mut value = 0u32;
            for i in 0..4 {
                let pa = Self::translate_data(cpu, addr + i, false)?;
                value |= (cpu.bus.read8(pa)? as u32) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn read64(cpu: &mut Cpu, addr: u64) -> Result<u64, Trap> {
        if (addr & 0xFFF) + 8 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, false)?;
            cpu.bus.read64(pa)
        } else {
            let mut value = 0u64;
            for i in 0..8 {
                let pa = Self::translate_data(cpu, addr + i, false)?;
                value |= (cpu.bus.read8(pa)? as u64) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn write8(cpu: &mut Cpu, addr: u64, value: u8) -> Result<(), Trap> {
        let pa = Self::translate_data(cpu, addr, true)?;
        cpu.bus.write8(pa, value)
    }

    pub fn write16(cpu: &mut Cpu, addr: u64, value: u16) -> Result<(), Trap> {
        if (addr & 0xFFF) + 2 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, true)?;
            cpu.bus.write16(pa, value)
        } else {
            for i in 0..2 {
                let pa = Self::translate_data(cpu, addr + i, true)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }

    pub fn write32(cpu: &mut Cpu, addr: u64, value: u32) -> Result<(), Trap> {
        if (addr & 0xFFF) + 4 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, true)?;
            cpu.bus.write32(pa, value)
        } else {
            for i in 0..4 {
                let pa = Self::translate_data(cpu, addr + i, true)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }

    pub fn write64(cpu: &mut Cpu, addr: u64, value: u64) -> Result<(), Trap> {
        if (addr & 0xFFF) + 8 <= 0x1000 {
            let pa = Self::translate_data(cpu, addr, true)?;
            cpu.bus.write64(pa, value)
        } else {
            for i in 0..8 {
                let pa = Self::translate_data(cpu, addr + i, true)?;
                cpu.bus.write8(pa, ((value >> (i * 8)) & 0xFF) as u8)?;
            }
            Ok(())
        }
    }

    pub fn fetch16(cpu: &mut Cpu, addr: u64) -> Result<u16, Trap> {
        if (addr & 0xFFF) + 2 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Instruction)?;
            cpu.bus.read16(pa)
        } else {
            let mut value = 0u16;
            for i in 0..2 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Instruction)?;
                value |= (cpu.bus.read8(pa)? as u16) << (i * 8);
            }
            Ok(value)
        }
    }

    pub fn fetch32(cpu: &mut Cpu, addr: u64) -> Result<u32, Trap> {
        if (addr & 0xFFF) + 4 <= 0x1000 {
            let pa = Mmu::translate(cpu, addr, AccessType::Instruction)?;
            cpu.bus.read32(pa)
        } else {
            let mut value = 0u32;
            for i in 0..4 {
                let pa = Mmu::translate(cpu, addr + i, AccessType::Instruction)?;
                value |= (cpu.bus.read8(pa)? as u32) << (i * 8);
            }
            Ok(value)
        }
    }
}
