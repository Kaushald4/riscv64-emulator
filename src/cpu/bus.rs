use crate::{cpu::memory::Memory, trap::Trap};

pub const RAM_BASE: u64 = 0x8000_0000;
pub const TOHOST_ADDR: u64 = 0x8000_1000;
pub const FROMHOST_ADDR: u64 = 0x8000_1008;

pub struct Bus {
    ram: Memory,

    pub tohost: Option<u64>,
}

impl Bus {
    pub fn new(ram_size: usize) -> Self {
        Self { ram: Memory::new(ram_size), tohost: None }
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

    #[inline]
    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
        self.ram.read8(offset)
    }

    #[inline]
    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
        self.ram.read16(offset)
    }

    #[inline]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
        self.ram.read32(offset)
    }

    #[inline]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let offset = self.ram_offset(addr, Trap::LoadAccessFault)?;
        self.ram.read64(offset)
    }

    #[inline]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.write8(offset, value)
    }

    #[inline]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.write16(offset, value)
    }

    #[inline]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.write32(offset, value)
    }

    #[inline]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        // for riscv test
        if addr == TOHOST_ADDR {
            self.tohost = Some(value);
            return Ok(());
        }

        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.write64(offset, value)
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault)?;
        self.ram.load(offset, bytes)
    }
}
