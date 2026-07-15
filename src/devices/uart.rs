use crate::trap::Trap;
use std::io::{self, Write};

pub const UART_BASE: u64 = 0x1000_0000;
pub const UART_SIZE: u64 = 0x100;

#[derive(Debug, Default)]
pub struct Uart {
    dll: u8,
    dlm: u8,
    ier: u8,
    fcr: u8,
    lcr: u8,
    mcr: u8,
    lsr: u8,
    msr: u8,
    scr: u8,
    thr: u8,
    rbr: u8,
}

impl Uart {
    pub fn new() -> Self {
        Self { lsr: 0x60, ..Default::default() }
    }

    #[inline]
    fn offset(addr: u64) -> Result<u64, Trap> {
        if !(UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return Err(Trap::LoadAccessFault);
        }

        Ok(addr - UART_BASE)
    }

    pub fn read8(&self, addr: u64) -> Result<u8, Trap> {
        let off = Self::offset(addr)?;

        let value = match off {
            0 => {
                if self.lcr & 0x80 != 0 {
                    self.dll
                } else {
                    self.rbr
                }
            }

            1 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm
                } else {
                    self.ier
                }
            }

            2 => 0x01, // IIR: no interrupt pending

            3 => self.lcr,
            4 => self.mcr,
            5 => self.lsr,
            6 => self.msr,
            7 => self.scr,

            _ => 0,
        };

        Ok(value)
    }

    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let off = Self::offset(addr)?;

        match off {
            0 => {
                if self.lcr & 0x80 != 0 {
                    self.dll = value;
                } else {
                    self.thr = value;
                    print!("{}", value as char);
                    io::stdout().flush().unwrap();
                }
            }

            1 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm = value;
                } else {
                    self.ier = value;
                }
            }

            2 => self.fcr = value,
            3 => self.lcr = value,
            4 => self.mcr = value,
            7 => self.scr = value,

            _ => {}
        }

        Ok(())
    }

    pub fn read16(&self, addr: u64) -> Result<u16, Trap> {
        Ok(self.read8(addr)? as u16 | ((self.read8(addr + 1)? as u16) << 8))
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
