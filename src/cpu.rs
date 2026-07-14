pub mod bus;
pub mod execute;
pub mod f_register;
pub mod memory;
pub mod register;

use crate::{cpu::bus::Bus, decode::decode, trap::Trap};

use f_register::FRegister;
use register::Register;

pub struct Cpu {
    pub regs: Register,
    pub f_regs: FRegister,
    pub pc: u64,
    pub bus: Bus,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Register::new(),
            f_regs: FRegister::new(),
            pc: 0x8000_0000,
            bus: Bus::new(128 * 1024 * 1024),
        }
    }

    pub fn step(&mut self, raw: u32) -> Result<(), Trap> {
        let decoded = decode(raw);

        execute::execute(decoded, self)?;

        self.pc = self.pc.wrapping_add(decoded.length as u64);

        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
