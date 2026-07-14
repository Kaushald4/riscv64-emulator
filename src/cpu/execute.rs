use crate::{
    cpu::Cpu,
    instruction::{DecodedInstruction, Instruction},
};

pub mod rv64i;
pub fn execute(decoded: DecodedInstruction, cpu: &mut Cpu) {
    match decoded.instruction {
        Instruction::Addi { rd, rs1, imm } => rv64i::addi(cpu, rd, rs1, imm),
        _ => unreachable!(),
    }
}
