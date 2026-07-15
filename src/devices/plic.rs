use crate::trap::Trap;

pub const PLIC_BASE: u64 = 0x0C00_0000;
pub const PLIC_SIZE: u64 = 0x0400_0000;

#[derive(Debug)]
pub struct Plic {
    memory: Vec<u8>,
}

impl Plic {
    pub fn new() -> Self {
        Self { memory: vec![0; PLIC_SIZE as usize] }
    }

    #[inline]
    fn offset(addr: u64) -> Result<usize, Trap> {
        if !(PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return Err(Trap::LoadAccessFault);
        }

        Ok((addr - PLIC_BASE) as usize)
    }

    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        Ok(self.memory[Self::offset(addr)?])
    }

    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let off = Self::offset(addr)?;
        self.memory[off] = value;
        Ok(())
    }

    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        let mut value = 0u16;

        for i in 0..2 {
            value |= (self.read8(addr + i)? as u16) << (i * 8);
        }

        Ok(value)
    }

    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let mut value = 0u32;

        for i in 0..4 {
            value |= (self.read8(addr + i)? as u32) << (i * 8);
        }

        Ok(value)
    }

    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let mut value = 0u64;

        for i in 0..8 {
            value |= (self.read8(addr + i)? as u64) << (i * 8);
        }

        Ok(value)
    }

    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        for i in 0..2 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }

    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        for i in 0..4 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }

    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        for i in 0..8 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }
}
