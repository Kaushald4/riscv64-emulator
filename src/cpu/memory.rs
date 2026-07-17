use crate::trap::Trap;

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }

    pub fn read_bulk(&self, addr: u64, buffer: &mut [u8]) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get(addr..addr + buffer.len()).ok_or(Trap::LoadAccessFault(addr as u64))?;
        buffer.copy_from_slice(slice);
        Ok(())
    }

    #[inline(always)]
    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let addr = addr as usize;
        self.data.get(addr).copied().ok_or(Trap::LoadAccessFault(addr as u64))
    }

    #[inline(always)]
    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        let addr = addr as usize;
        let slice = self.data.get(addr..addr + 2).ok_or(Trap::LoadAccessFault(addr as u64))?;
        Ok(u16::from_le_bytes(slice.try_into().unwrap()))
    }

    #[inline(always)]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let addr = addr as usize;
        let slice = self.data.get(addr..addr + 4).ok_or(Trap::LoadAccessFault(addr as u64))?;
        Ok(u32::from_le_bytes(slice.try_into().unwrap()))
    }

    #[inline(always)]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let addr = addr as usize;
        let slice = self.data.get(addr..addr + 8).ok_or(Trap::LoadAccessFault(addr as u64))?;
        Ok(u64::from_le_bytes(slice.try_into().unwrap()))
    }

    #[inline(always)]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let addr = addr as usize;

        let byte = self.data.get_mut(addr).ok_or(Trap::StoreAccessFault(addr as u64))?;
        *byte = value;
        Ok(())
    }

    #[inline(always)]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get_mut(addr..addr + 2).ok_or(Trap::StoreAccessFault(addr as u64))?;
        slice.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get_mut(addr..addr + 4).ok_or(Trap::StoreAccessFault(addr as u64))?;
        slice.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    #[inline(always)]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get_mut(addr..addr + 8).ok_or(Trap::StoreAccessFault(addr as u64))?;
        slice.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get_mut(addr..addr + bytes.len()).ok_or(Trap::StoreAccessFault(addr as u64))?;
        slice.copy_from_slice(bytes);
        Ok(())
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
