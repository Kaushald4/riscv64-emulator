use crate::register::Reg;

#[derive(Debug)]
pub enum Instruction {
    // Type I RV32I ALU immediate instructions
    Addi { rd: Reg, rs1: Reg, imm: i64 },
    Slti { rd: Reg, rs1: Reg, imm: i64 },
    Sltiu { rd: Reg, rs1: Reg, imm: i64 },
    Xori { rd: Reg, rs1: Reg, imm: i64 },
    Ori { rd: Reg, rs1: Reg, imm: i64 },
    Andi { rd: Reg, rs1: Reg, imm: i64 },
    Slli { rd: Reg, rs1: Reg, shamt: i64 },
    Srli { rd: Reg, rs1: Reg, shamt: i64 },
    Srai { rd: Reg, rs1: Reg, shamt: i64 },
    // RV64I word instructions
    Addiw { rd: Reg, rs1: Reg, imm: i64 },
    Slliw { rd: Reg, rs1: Reg, shamt: i64 },
    Srliw { rd: Reg, rs1: Reg, shamt: i64 },
    Sraiw { rd: Reg, rs1: Reg, shamt: i64 },

    // Type R RV32I ALU register-register instructions
    Add { rd: Reg, rs1: Reg, rs2: Reg },
    Sub { rd: Reg, rs1: Reg, rs2: Reg },
    Sll { rd: Reg, rs1: Reg, rs2: Reg },
    Slt { rd: Reg, rs1: Reg, rs2: Reg },
    Sltu { rd: Reg, rs1: Reg, rs2: Reg },
    Sra { rd: Reg, rs1: Reg, rs2: Reg },
    Xor { rd: Reg, rs1: Reg, rs2: Reg },
    Srl { rd: Reg, rs1: Reg, rs2: Reg },
    Or { rd: Reg, rs1: Reg, rs2: Reg },
    And { rd: Reg, rs1: Reg, rs2: Reg },
    // RV32M  ALU register-register standard extensions
    Mul { rd: Reg, rs1: Reg, rs2: Reg },
    Mulh { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhsu { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhu { rd: Reg, rs1: Reg, rs2: Reg },
    Div { rd: Reg, rs1: Reg, rs2: Reg },
    Divu { rd: Reg, rs1: Reg, rs2: Reg },
    Rem { rd: Reg, rs1: Reg, rs2: Reg },
    Remu { rd: Reg, rs1: Reg, rs2: Reg },
    // RV64I word instructions
    Addw { rd: Reg, rs1: Reg, rs2: Reg },
    Subw { rd: Reg, rs1: Reg, rs2: Reg },
    Sllw { rd: Reg, rs1: Reg, rs2: Reg },
    Srlw { rd: Reg, rs1: Reg, rs2: Reg },
    Sraw { rd: Reg, rs1: Reg, rs2: Reg },
    // RV64M word multiply/divide instructions
    Mulw { rd: Reg, rs1: Reg, rs2: Reg },
    Divw { rd: Reg, rs1: Reg, rs2: Reg },
    Divuw { rd: Reg, rs1: Reg, rs2: Reg },
    Remw { rd: Reg, rs1: Reg, rs2: Reg },
    Remuw { rd: Reg, rs1: Reg, rs2: Reg },

    Undefined { raw: u32 },
}
