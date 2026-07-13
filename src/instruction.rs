use crate::register::Reg;

#[derive(Debug)]
pub enum Instruction {
    Addi { rd: Reg, rs1: Reg, imm: i64 },
    Slti { rd: Reg, rs1: Reg, imm: i64 },
    Sltiu { rd: Reg, rs1: Reg, imm: i64 },
    Xori { rd: Reg, rs1: Reg, imm: i64 },
    Ori { rd: Reg, rs1: Reg, imm: i64 },
    Andi { rd: Reg, rs1: Reg, imm: i64 },
    Slli { rd: Reg, rs1: Reg, shamt: i64 },
    Srli { rd: Reg, rs1: Reg, shamt: i64 },
    Srai { rd: Reg, rs1: Reg, shamt: i64 },

    Undefined { raw: u32 },
}
