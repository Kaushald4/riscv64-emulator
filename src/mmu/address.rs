#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn offset(self) -> u16 {
        (self.0 & 0xfff) as u16
    }

    #[inline]
    pub const fn vpn0(self) -> u16 {
        ((self.0 >> 12) & 0x1ff) as u16
    }

    #[inline]
    pub const fn vpn1(self) -> u16 {
        ((self.0 >> 21) & 0x1ff) as u16
    }

    #[inline]
    pub const fn vpn2(self) -> u16 {
        ((self.0 >> 30) & 0x1ff) as u16
    }

    #[inline]
    pub const fn page_offset(self) -> u16 {
        self.offset()
    }

    pub const fn is_canonical(self) -> bool {
        let sign = (self.0 >> 38) & 1;
        let upper = self.0 >> 39;

        if sign == 0 { upper == 0 } else { upper == 0x1ffffff }
    }
}
