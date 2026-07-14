pub mod execute;
pub mod f_register;
pub mod register;

use crate::decode::decode;

use f_register::FRegister;
use register::Register;

pub struct Cpu {
    pub regs: Register,
    pub f_regs: FRegister,
    pub pc: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Self { regs: Register::new(), f_regs: FRegister::new(), pc: 0 }
    }

    pub fn step(&mut self, raw: u32) {
        let decoded = decode(raw);

        execute::execute(decoded, self);

        self.pc = self.pc.wrapping_add(decoded.length as u64);
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
