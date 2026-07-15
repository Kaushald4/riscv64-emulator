#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pte(u64);

impl Pte {
    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    // =========================================================================
    // Flags
    // =========================================================================

    #[inline]
    pub const fn valid(self) -> bool {
        self.0 & (1 << 0) != 0
    }

    #[inline]
    pub const fn read(self) -> bool {
        self.0 & (1 << 1) != 0
    }

    #[inline]
    pub const fn write(self) -> bool {
        self.0 & (1 << 2) != 0
    }

    #[inline]
    pub const fn execute(self) -> bool {
        self.0 & (1 << 3) != 0
    }

    #[inline]
    pub const fn user(self) -> bool {
        self.0 & (1 << 4) != 0
    }

    #[inline]
    pub const fn global(self) -> bool {
        self.0 & (1 << 5) != 0
    }

    #[inline]
    pub const fn accessed(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    #[inline]
    pub const fn dirty(self) -> bool {
        self.0 & (1 << 7) != 0
    }

    // =========================================================================
    // Physical Page Number
    // =========================================================================

    #[inline]
    pub const fn ppn0(self) -> u16 {
        ((self.0 >> 10) & 0x1ff) as u16
    }

    #[inline]
    pub const fn ppn1(self) -> u16 {
        ((self.0 >> 19) & 0x1ff) as u16
    }

    #[inline]
    pub const fn ppn2(self) -> u32 {
        ((self.0 >> 28) & 0x03ff_ffff) as u32
    }

    #[inline]
    pub const fn ppn(self) -> u64 {
        (self.0 >> 10) & ((1u64 << 44) - 1)
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Invalid if:
    /// - V == 0
    /// - R == 0 && W == 1
    #[inline]
    pub const fn is_invalid(self) -> bool {
        !self.valid() || (!self.read() && self.write())
    }

    /// Leaf if either R or X is set.
    #[inline]
    pub const fn is_leaf(self) -> bool {
        self.read() || self.execute()
    }
}
