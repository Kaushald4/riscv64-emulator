use std::time::Instant;

use crate::trap::Trap;

pub const CLINT_BASE: u64 = 0x0200_0000;
pub const CLINT_SIZE: u64 = 0x0001_0000;

const MSIP_OFFSET: u64 = 0x0000;
const MTIMECMP_OFFSET: u64 = 0x4000;
const MTIME_OFFSET: u64 = 0xBFF8;

#[derive(Debug)]
pub struct Clint {
    pub msip: u32,
    pub mtimecmp: u64,
    pub mtime: u64,

    start_time: Instant,
    pub frequency: u64,
}

impl Clint {
    pub fn new() -> Self {
        Self {
            msip: 0,
            mtimecmp: u64::MAX,
            mtime: 0,

            start_time: Instant::now(),
            frequency: 10_000_000, // 10 MHz
        }
    }

    #[inline]
    fn offset(addr: u64) -> Result<u64, Trap> {
        if !(CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return Err(Trap::LoadAccessFault(addr));
        }

        Ok(addr - CLINT_BASE)
    }

    #[inline]
    pub fn tick(&mut self) {
        self.mtime = self.mtime.wrapping_add(1);
    }
    // #[inline]
    // pub fn tick(&mut self) {
    //     let elapsed_ns = self.start_time.elapsed().as_nanos() as u64;
    //     self.mtime = elapsed_ns * self.frequency / 1_000_000_000;
    // }

    #[inline]
    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let offset = Self::offset(addr)?;

        let value = match offset {
            x if (MSIP_OFFSET..MSIP_OFFSET + 4).contains(&x) => {
                let shift = (offset - MSIP_OFFSET) * 8;
                ((self.msip >> shift) & 0xff) as u8
            }

            x if (MTIMECMP_OFFSET..MTIMECMP_OFFSET + 8).contains(&x) => {
                let shift = (offset - MTIMECMP_OFFSET) * 8;
                ((self.mtimecmp >> shift) & 0xff) as u8
            }

            x if (MTIME_OFFSET..MTIME_OFFSET + 8).contains(&x) => {
                let shift = (offset - MTIME_OFFSET) * 8;
                ((self.mtime >> shift) & 0xff) as u8
            }

            _ => return Err(Trap::LoadAccessFault(addr)),
        };

        Ok(value)
    }

    #[inline]
    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        let mut value = 0u16;

        for i in 0..2 {
            value |= (self.read8(addr + i)? as u16) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    pub fn read32(&self, addr: u64) -> Result<u32, Trap> {
        let mut value = 0u32;

        for i in 0..4 {
            value |= (self.read8(addr + i)? as u32) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    pub fn read64(&self, addr: u64) -> Result<u64, Trap> {
        let mut value = 0u64;

        for i in 0..8 {
            value |= (self.read8(addr + i)? as u64) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let offset = Self::offset(addr)?;

        match offset {
            x if (MSIP_OFFSET..MSIP_OFFSET + 4).contains(&x) => {
                let shift = (offset - MSIP_OFFSET) * 8;
                self.msip &= !(0xff << shift);
                self.msip |= (value as u32) << shift;
            }

            x if (MTIMECMP_OFFSET..MTIMECMP_OFFSET + 8).contains(&x) => {
                let shift = (offset - MTIMECMP_OFFSET) * 8;
                self.mtimecmp &= !(0xffu64 << shift);
                self.mtimecmp |= (value as u64) << shift;
            }

            x if (MTIME_OFFSET..MTIME_OFFSET + 8).contains(&x) => {
                let shift = (offset - MTIME_OFFSET) * 8;
                self.mtime &= !(0xffu64 << shift);
                self.mtime |= (value as u64) << shift;
            }

            _ => return Err(Trap::StoreAccessFault(addr)),
        }

        Ok(())
    }

    #[inline]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        for i in 0..2 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }

    #[inline]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        for i in 0..4 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }

    #[inline]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        for i in 0..8 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }

        Ok(())
    }
}
