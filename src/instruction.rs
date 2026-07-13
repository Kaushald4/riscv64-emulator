use crate::register::Reg;

#[derive(Debug)]
pub enum Instruction {
    Addi { rd: Reg, rs1: Reg, imm: i64 },

    Undefined { raw: u32 },
}
