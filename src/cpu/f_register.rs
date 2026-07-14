use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FReg(u8);

impl FReg {
    #[inline]
    pub const fn new(bits: u32) -> Self {
        FReg((bits & 0b11111) as u8)
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    // RISC-V floating-point ABI names
    pub const fn abi_name(self) -> &'static str {
        match self.0 {
            0 => "ft0",
            1 => "ft1",
            2 => "ft2",
            3 => "ft3",
            4 => "ft4",
            5 => "ft5",
            6 => "ft6",
            7 => "ft7",
            8 => "fs0",
            9 => "fs1",
            10 => "fa0",
            11 => "fa1",
            12 => "fa2",
            13 => "fa3",
            14 => "fa4",
            15 => "fa5",
            16 => "fa6",
            17 => "fa7",
            18 => "fs2",
            19 => "fs3",
            20 => "fs4",
            21 => "fs5",
            22 => "fs6",
            23 => "fs7",
            24 => "fs8",
            25 => "fs9",
            26 => "fs10",
            27 => "fs11",
            28 => "ft8",
            29 => "ft9",
            30 => "ft10",
            31 => "ft11",
            _ => unreachable!(),
        }
    }
}

pub struct FRegister {
    f: [u64; 32],
}

impl FRegister {
    pub fn new() -> Self {
        Self { f: [0; 32] }
    }

    #[inline]
    pub fn read(&self, r: FReg) -> u64 {
        self.f[r.idx()]
    }

    #[inline]
    pub fn write(&mut self, r: FReg, val: u64) {
        self.f[r.idx()] = val;
    }

    pub fn dump(&self) {
        for i in 0..32 {
            let r = FReg::new(i);
            println!("{:<5}=0x{:016x}", r.abi_name(), self.f[r.idx()]);
            if i % 4 == 3 {
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
