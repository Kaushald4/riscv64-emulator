use crate::trap::Trap;

pub const CSR_MHARTID: u16 = 0xF14;

pub struct Csr {
    mhartid: u64,
}

impl Csr {
    pub fn new() -> Self {
        Self { mhartid: 0 }
    }

    #[inline]
    pub fn read(&self, csr: u16) -> Result<u64, Trap> {
        match csr {
            CSR_MHARTID => Ok(self.mhartid),

            _ => Err(Trap::IllegalInstruction(csr as u32)),
        }
    }

    #[inline]
    pub fn write(&mut self, csr: u16, value: u64) -> Result<(), Trap> {
        match csr {
            CSR_MHARTID => Err(Trap::IllegalInstruction(csr as u32)),

            _ => Err(Trap::IllegalInstruction(csr as u32)),
        }
    }
}

impl Default for Csr {
    fn default() -> Self {
        Self::new()
    }
}
