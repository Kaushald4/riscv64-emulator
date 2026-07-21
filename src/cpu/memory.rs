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
        if addr + 2 <= self.data.len() {
            let ptr = self.data.as_ptr().wrapping_add(addr) as *const u16;
            Ok(unsafe { ptr.read_unaligned() })
        } else {
            Err(Trap::LoadAccessFault(addr as u64))
        }
    }

    #[inline(always)]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let addr = addr as usize;
        if addr + 4 <= self.data.len() {
            let ptr = self.data.as_ptr().wrapping_add(addr) as *const u32;
            Ok(unsafe { ptr.read_unaligned() })
        } else {
            Err(Trap::LoadAccessFault(addr as u64))
        }
    }

    #[inline(always)]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let addr = addr as usize;
        if addr + 8 <= self.data.len() {
            let ptr = self.data.as_ptr().wrapping_add(addr) as *const u64;
            Ok(unsafe { ptr.read_unaligned() })
        } else {
            Err(Trap::LoadAccessFault(addr as u64))
        }
    }

    #[inline(always)]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let addr = addr as usize;
        self.data.get_mut(addr).map(|v| *v = value).ok_or(Trap::StoreAccessFault(addr as u64))
    }

    #[inline(always)]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        let addr = addr as usize;
        if addr + 2 <= self.data.len() {
            let ptr = self.data.as_mut_ptr().wrapping_add(addr) as *mut u16;
            unsafe { ptr.write_unaligned(value) };
            Ok(())
        } else {
            Err(Trap::StoreAccessFault(addr as u64))
        }
    }

    #[inline(always)]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        let addr = addr as usize;
        if addr + 4 <= self.data.len() {
            let ptr = self.data.as_mut_ptr().wrapping_add(addr) as *mut u32;
            unsafe { ptr.write_unaligned(value) };
            Ok(())
        } else {
            Err(Trap::StoreAccessFault(addr as u64))
        }
    }

    #[inline(always)]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        let addr = addr as usize;
        if addr + 8 <= self.data.len() {
            let ptr = self.data.as_mut_ptr().wrapping_add(addr) as *mut u64;
            unsafe { ptr.write_unaligned(value) };
            Ok(())
        } else {
            Err(Trap::StoreAccessFault(addr as u64))
        }
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let addr = addr as usize;
        let slice = self.data.get_mut(addr..addr + bytes.len()).ok_or(Trap::StoreAccessFault(addr as u64))?;
        slice.copy_from_slice(bytes);
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn read8_unchecked(&self, addr: u64) -> u8 {
        let addr = addr as usize;
        unsafe { *self.data.get_unchecked(addr) }
    }

    #[inline(always)]
    pub unsafe fn read16_unchecked(&self, addr: u64) -> u16 {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_ptr().add(addr) } as *const u16;
        unsafe { ptr.read_unaligned() }
    }

    #[inline(always)]
    pub unsafe fn read32_unchecked(&self, addr: u64) -> u32 {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_ptr().add(addr) } as *const u32;
        unsafe { ptr.read_unaligned() }
    }

    #[inline(always)]
    pub unsafe fn read64_unchecked(&self, addr: u64) -> u64 {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_ptr().add(addr) } as *const u64;
        unsafe { ptr.read_unaligned() }
    }

    #[inline(always)]
    pub unsafe fn write8_unchecked(&mut self, addr: u64, value: u8) {
        let addr = addr as usize;
        unsafe { *self.data.get_unchecked_mut(addr) = value };
    }

    #[inline(always)]
    pub unsafe fn write16_unchecked(&mut self, addr: u64, value: u16) {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_mut_ptr().add(addr) } as *mut u16;
        unsafe { ptr.write_unaligned(value) };
    }

    #[inline(always)]
    pub unsafe fn write32_unchecked(&mut self, addr: u64, value: u32) {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_mut_ptr().add(addr) } as *mut u32;
        unsafe { ptr.write_unaligned(value) };
    }

    #[inline(always)]
    pub unsafe fn write64_unchecked(&mut self, addr: u64, value: u64) {
        let addr = addr as usize;
        let ptr = unsafe { self.data.as_mut_ptr().add(addr) } as *mut u64;
        unsafe { ptr.write_unaligned(value) };
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}
