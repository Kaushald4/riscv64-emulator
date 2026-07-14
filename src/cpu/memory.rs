use crate::trap::Trap;

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }

    #[inline]
    fn check(&self, addr: usize, size: usize, trap: Trap) -> Result<(), Trap> {
        match addr.checked_add(size) {
            Some(end) if end <= self.data.len() => Ok(()),
            _ => Err(trap),
        }
    }

    #[inline]
    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let addr = addr as usize;

        self.check(addr, 1, Trap::LoadAccessFault)?;

        Ok(self.data[addr])
    }

    #[inline]
    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        let addr = addr as usize;

        self.check(addr, 2, Trap::LoadAccessFault)?;

        Ok(u16::from_le_bytes([self.data[addr], self.data[addr + 1]]))
    }

    #[inline]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let addr = addr as usize;

        self.check(addr, 4, Trap::LoadAccessFault)?;

        Ok(u32::from_le_bytes([self.data[addr], self.data[addr + 1], self.data[addr + 2], self.data[addr + 3]]))
    }

    #[inline]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let addr = addr as usize;

        self.check(addr, 8, Trap::LoadAccessFault)?;

        Ok(u64::from_le_bytes([self.data[addr], self.data[addr + 1], self.data[addr + 2], self.data[addr + 3], self.data[addr + 4], self.data[addr + 5], self.data[addr + 6], self.data[addr + 7]]))
    }

    #[inline]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let addr = addr as usize;

        self.check(addr, 1, Trap::StoreAccessFault)?;

        self.data[addr] = value;

        Ok(())
    }

    #[inline]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        let addr = addr as usize;

        self.check(addr, 2, Trap::StoreAccessFault)?;

        self.data[addr..addr + 2].copy_from_slice(&value.to_le_bytes());

        Ok(())
    }

    #[inline]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        let addr = addr as usize;

        self.check(addr, 4, Trap::StoreAccessFault)?;

        self.data[addr..addr + 4].copy_from_slice(&value.to_le_bytes());

        Ok(())
    }

    #[inline]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        let addr = addr as usize;

        self.check(addr, 8, Trap::StoreAccessFault)?;

        self.data[addr..addr + 8].copy_from_slice(&value.to_le_bytes());

        Ok(())
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let addr = addr as usize;

        self.check(addr, bytes.len(), Trap::StoreAccessFault)?;

        self.data[addr..addr + bytes.len()].copy_from_slice(bytes);

        Ok(())
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
