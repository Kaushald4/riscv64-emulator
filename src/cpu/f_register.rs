use std::fmt;

const FREG_ABI_NAMES: [&str; 32] = [
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", "fs0", "fs1", "fa0", "fa1", "fa2", "fa3", "fa4", "fa5", "fa6", "fa7", "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9", "fs10", "fs11", "ft8", "ft9", "ft10", "ft11",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FReg(u8);

impl FReg {
    #[inline]
    pub const fn new(bits: u32) -> Self {
        Self((bits & 0b1_1111) as u8)
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn abi_name(self) -> &'static str {
        FREG_ABI_NAMES[self.0 as usize]
    }
}

pub struct FRegister {
    regs: [u64; 32],
}

impl FRegister {
    /// upper 32 bits used for NaN-boxing RV32F values on RV64.
    const NAN_BOX_UPPER: u64 = 0xffff_ffff_0000_0000;

    /// canonical quiet NaN (single precision).
    const CANONICAL_NAN_F32: u32 = 0x7fc0_0000;

    pub fn new() -> Self {
        Self { regs: [0; 32] }
    }

    // raw register access
    #[inline]
    pub fn read_bits(&self, reg: FReg) -> u64 {
        self.regs[reg.idx()]
    }

    #[inline]
    pub fn write_bits(&mut self, reg: FReg, bits: u64) {
        self.regs[reg.idx()] = bits;
    }

    // RV64F (Single Precision)
    #[inline]
    pub fn read_f32_bits(&self, reg: FReg) -> u32 {
        let value = self.regs[reg.idx()];

        if (value >> 32) == 0xffff_ffff { value as u32 } else { Self::CANONICAL_NAN_F32 }
    }

    #[inline]
    pub fn write_f32_bits(&mut self, reg: FReg, bits: u32) {
        self.regs[reg.idx()] = Self::NAN_BOX_UPPER | (bits as u64);
    }

    #[inline]
    pub fn is_nan_boxed(&self, reg: FReg) -> bool {
        (self.regs[reg.idx()] >> 32) == 0xffff_ffff
    }

    #[inline]
    pub fn read_f32_raw_bits(&self, reg: FReg) -> u32 {
        self.read_bits(reg) as u32
    }

    // RV64D (Double Precision)
    #[inline]
    pub fn read_f64_bits(&self, reg: FReg) -> u64 {
        self.regs[reg.idx()]
    }

    #[inline]
    pub fn write_f64_bits(&mut self, reg: FReg, bits: u64) {
        self.regs[reg.idx()] = bits;
    }

    // for debug
    pub fn dump(&self) {
        const COLS: usize = 4;

        for i in 0..32 {
            let reg = FReg::new(i);

            print!("{:<5}=0x{:016x}  ", reg.abi_name(), self.regs[reg.idx()]);

            if (i as usize + 1) % COLS == 0 {
                println!();
            }
        }
    }
}

impl Default for FRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for FReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "f{}({})", self.0, self.abi_name())
    }
}

impl fmt::Display for FReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abi_name())
    }
}
