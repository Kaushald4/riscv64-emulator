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
    Slli { rd: Reg, rs1: Reg, shamt: u32 },
    Srli { rd: Reg, rs1: Reg, shamt: u32 },
    Srai { rd: Reg, rs1: Reg, shamt: u32 },
    // Type I RV64I word immediate instructions
    Addiw { rd: Reg, rs1: Reg, imm: i64 },
    Slliw { rd: Reg, rs1: Reg, shamt: u32 },
    Srliw { rd: Reg, rs1: Reg, shamt: u32 },
    Sraiw { rd: Reg, rs1: Reg, shamt: u32 },

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
    // Type R RV32M multiply/divide instructions
    Mul { rd: Reg, rs1: Reg, rs2: Reg },
    Mulh { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhsu { rd: Reg, rs1: Reg, rs2: Reg },
    Mulhu { rd: Reg, rs1: Reg, rs2: Reg },
    Div { rd: Reg, rs1: Reg, rs2: Reg },
    Divu { rd: Reg, rs1: Reg, rs2: Reg },
    Rem { rd: Reg, rs1: Reg, rs2: Reg },
    Remu { rd: Reg, rs1: Reg, rs2: Reg },
    // Type R RV64I word instructions
    Addw { rd: Reg, rs1: Reg, rs2: Reg },
    Subw { rd: Reg, rs1: Reg, rs2: Reg },
    Sllw { rd: Reg, rs1: Reg, rs2: Reg },
    Srlw { rd: Reg, rs1: Reg, rs2: Reg },
    Sraw { rd: Reg, rs1: Reg, rs2: Reg },
    // Type R RV64M word multiply/divide instructions
    Mulw { rd: Reg, rs1: Reg, rs2: Reg },
    Divw { rd: Reg, rs1: Reg, rs2: Reg },
    Divuw { rd: Reg, rs1: Reg, rs2: Reg },
    Remw { rd: Reg, rs1: Reg, rs2: Reg },
    Remuw { rd: Reg, rs1: Reg, rs2: Reg },

    // Type I RV32I load instructions
    Lb { rd: Reg, rs1: Reg, imm: i64 },
    Lh { rd: Reg, rs1: Reg, imm: i64 },
    Lw { rd: Reg, rs1: Reg, imm: i64 },
    Lbu { rd: Reg, rs1: Reg, imm: i64 },
    Lhu { rd: Reg, rs1: Reg, imm: i64 },
    // Type I RV64I load instructions
    Lwu { rd: Reg, rs1: Reg, imm: i64 },
    Ld { rd: Reg, rs1: Reg, imm: i64 },

    // Type S RV32I store instructions
    Sb { rs2: Reg, rs1: Reg, imm: i64 },
    Sh { rs2: Reg, rs1: Reg, imm: i64 },
    Sw { rs2: Reg, rs1: Reg, imm: i64 },
    // Type S RV64I store instructions
    Sd { rs2: Reg, rs1: Reg, imm: i64 },

    // RV32A atomic instructions
    Lrw { rd: Reg, rs1: Reg, rl: bool, aq: bool },
    Scw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoswapw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoaddw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoxorw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoandw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoorw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amominw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amomaxw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amominuw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amomaxuw { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    // RV64A atomic instructions
    Lrd { rd: Reg, rs1: Reg, rl: bool, aq: bool },
    Scd { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoswapd { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoaddd { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoxord { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoandd { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amoord { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amomind { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amomaxd { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amominud { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },
    Amomaxud { rd: Reg, rs1: Reg, rs2: Reg, rl: bool, aq: bool },

    // Type B branch instructions
    Beq { rs1: Reg, rs2: Reg, imm: i64 },
    Bne { rs1: Reg, rs2: Reg, imm: i64 },
    Blt { rs1: Reg, rs2: Reg, imm: i64 },
    Bge { rs1: Reg, rs2: Reg, imm: i64 },
    Bltu { rs1: Reg, rs2: Reg, imm: i64 },
    Bgeu { rs1: Reg, rs2: Reg, imm: i64 },

    Undefined { raw: u32 },
}
