use crate::{cpu::memory::Memory, trap::Trap};

pub const RAM_BASE: u64 = 0x8000_0000;

#[derive(Debug, Clone, Copy)]
pub enum MisalignedAccess {
    Trap,
    Emulate,
}

pub struct Bus {
    ram: Memory,
    misaligned: MisalignedAccess,
}

impl Bus {
    pub fn new(ram_size: usize) -> Self {
        Self {
            ram: Memory::new(ram_size),
            misaligned: MisalignedAccess::Emulate,
        }
    }

    #[inline]
    fn ram_offset(&self, addr: u64, trap: Trap) -> Result<u64, Trap> {
        if addr < RAM_BASE {
            return Err(trap);
        }

        let offset = addr - RAM_BASE;

        if offset >= self.ram.size() as u64 {
            return Err(trap);
        }

        Ok(offset)
    }

    // reads
    #[inline]
    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
        self.ram.read8(offset)
    }

    #[inline]
    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 1 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                self.ram.read16(offset)
            }

            MisalignedAccess::Emulate => {
                if addr & 1 == 0 {
                    let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                    self.ram.read16(offset)
                } else {
                    self.read16_emulated(addr)
                }
            }
        }
    }

    #[inline]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 3 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                self.ram.read32(offset)
            }

            MisalignedAccess::Emulate => {
                if addr & 3 == 0 {
                    let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                    self.ram.read32(offset)
                } else {
                    self.read32_emulated(addr)
                }
            }
        }
    }

    #[inline]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 7 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                self.ram.read64(offset)
            }

            MisalignedAccess::Emulate => {
                if addr & 7 == 0 {
                    let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
                    self.ram.read64(offset)
                } else {
                    self.read64_emulated(addr)
                }
            }
        }
    }

    // writes
    #[inline]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.write8(offset, value)
    }

    #[inline]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 1 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                self.ram.write16(offset, value)
            }

            MisalignedAccess::Emulate => {
                if addr & 1 == 0 {
                    let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                    self.ram.write16(offset, value)
                } else {
                    self.write16_emulated(addr, value)
                }
            }
        }
    }

    #[inline]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 3 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                self.ram.write32(offset, value)
            }

            MisalignedAccess::Emulate => {
                if addr & 3 == 0 {
                    let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                    self.ram.write32(offset, value)
                } else {
                    self.write32_emulated(addr, value)
                }
            }
        }
    }

    #[inline]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        match self.misaligned {
            MisalignedAccess::Trap => {
                if addr & 7 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                self.ram.write64(offset, value)
            }

            MisalignedAccess::Emulate => {
                if addr & 7 == 0 {
                    let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
                    self.ram.write64(offset, value)
                } else {
                    self.write64_emulated(addr, value)
                }
            }
        }
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.load(offset, bytes)
    }

    // private emulation helpers
    #[inline]
    fn read16_emulated(&self, addr: u64) -> Result<u16, Trap> {
        Ok((self.read8(addr)? as u16) | ((self.read8(addr + 1)? as u16) << 8))
    }

    #[inline]
    fn read32_emulated(&self, addr: u64) -> Result<u32, Trap> {
        let mut value = 0u32;

        for i in 0..4 {
            value |= (self.read8(addr + i)? as u32) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    fn read64_emulated(&self, addr: u64) -> Result<u64, Trap> {
        let mut value = 0u64;

        for i in 0..8 {
            value |= (self.read8(addr + i)? as u64) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    fn write16_emulated(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        self.write8(addr, value as u8)?;
        self.write8(addr + 1, (value >> 8) as u8)
    }

    #[inline]
    fn write32_emulated(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        for i in 0..4 {
            self.write8(addr + i, (value >> (i * 8)) as u8)?;
        }

        Ok(())
    }

    #[inline]
    fn write64_emulated(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        for i in 0..8 {
            self.write8(addr + i, (value >> (i * 8)) as u8)?;
        }

        Ok(())
    }
}
