#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatpMode {
    Bare,
    Sv39,
    Reserved(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Satp {
    bits: u64,
}

impl Satp {
    pub fn new(bits: u64) -> Self {
        Self { bits }
    }

    #[inline]
    pub fn bits(self) -> u64 {
        self.bits
    }

    #[inline]
    pub fn mode(self) -> SatpMode {
        match (self.bits >> 60) as u8 {
            0 => SatpMode::Bare,
            8 => SatpMode::Sv39,
            x => SatpMode::Reserved(x),
        }
    }

    #[inline]
    pub fn asid(self) -> u16 {
        ((self.bits >> 44) & 0xffff) as u16
    }

    #[inline]
    pub fn ppn(self) -> u64 {
        self.bits & ((1u64 << 44) - 1)
    }
}
