use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Reg(u8);

impl Reg {
    #[inline]
    pub const fn new(bits: u32) -> Self {
        Reg((bits & 0b11111) as u8)
    }

    #[inline]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    // riscv has special names for registers from x0..x31
    pub const fn abi_name(self) -> &'static str {
        match self.0 {
            0 => "zero",
            1 => "ra",
            2 => "sp",
            3 => "gp",
            4 => "tp",
            5 => "t0",
            6 => "t1",
            7 => "t2",
            8 => "s0",
            9 => "s1",
            10 => "a0",
            11 => "a1",
            12 => "a2",
            13 => "a3",
            14 => "a4",
            15 => "a5",
            16 => "a6",
            17 => "a7",
            18 => "s2",
            19 => "s3",
            20 => "s4",
            21 => "s5",
            22 => "s6",
            23 => "s7",
            24 => "s8",
            25 => "s9",
            26 => "s10",
            27 => "s11",
            28 => "t3",
            29 => "t4",
            30 => "t5",
            31 => "t6",
            _ => unreachable!(),
        }
    }
}

pub struct Register {
    x: [u64; 32],
}

impl Register {
    pub fn new() -> Self {
        Self { x: [0; 32] }
    }

    #[inline]
    pub fn read(&self, r: Reg) -> u64 {
        if r.idx() == 0 { 0 } else { self.x[r.idx()] }
    }

    #[inline]
    pub fn write(&mut self, r: Reg, val: u64) {
        if r.idx() != 0 {
            self.x[r.idx()] = val;
        }
    }

    pub fn dump(&self) {
        for i in 0..32 {
            let r = Reg::new(i);
            println!("{:<5}=0x{:061x} ", r.abi_name(), self.x[r.idx()]);
            if i % 4 == 3 {
                println!();
            }
        }
    }
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

// for debug
impl fmt::Debug for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{}({})", self.0, self.abi_name())
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abi_name())
    }
}
