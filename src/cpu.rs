pub mod bus;
pub mod csr;
pub mod execute;
pub mod f_register;
pub mod memory;
pub mod register;

use crate::{
    cpu::{bus::Bus, csr::Csr},
    decode::decode,
    trap::Trap,
};

use f_register::FRegister;
use register::Register;

#[derive(Debug)]
pub enum ExecFlow {
    Next,
    Jump(u64),
}
pub type ExecResult = Result<ExecFlow, Trap>;

pub enum PrivilegeMode {
    Machine,
    Supervisor,
    User,
}

pub struct Cpu {
    pub regs: Register,
    pub f_regs: FRegister,
    pub pc: u64,
    pub bus: Bus,
    pub current_instruction_length: u8,
    pub privilege_mode: PrivilegeMode,
    pub csr: Csr,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Register::new(),
            f_regs: FRegister::new(),
            pc: 0x8000_0000,
            bus: Bus::new(128 * 1024 * 1024),
            current_instruction_length: 4,
            privilege_mode: PrivilegeMode::Machine,
            csr: Csr::new(),
        }
    }

    fn fetch(&mut self) -> Result<u32, Trap> {
        let first = self.bus.read16(self.pc)?;

        if first & 0b11 != 0b11 {
            Ok(first as u32)
        } else {
            let second = self.bus.read16(self.pc + 2)?;

            Ok((first as u32) | ((second as u32) << 16))
        }
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        let raw = self.fetch()?;

        let decoded = decode(raw);

        let length = decoded.length;
        self.current_instruction_length = length;

        let flow = execute::execute(decoded, self)?;

        match flow {
            ExecFlow::Next => {
                self.pc = self.pc.wrapping_add(length as u64);
            }

            ExecFlow::Jump(target) => {
                self.pc = target;
            }
        }

        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
