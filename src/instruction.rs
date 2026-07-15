use crate::cpu::f_register::FReg;
use crate::cpu::register::Reg;

#[derive(Debug, Clone, Copy)]
pub struct DecodedInstruction {
    pub instruction: Instruction,
    pub length: u8,
}

#[derive(Debug, Clone, Copy)]
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

    // Type J Jump and link
    Jal { rd: Reg, imm: i64 },

    // Type I Jump and link register
    Jalr { rd: Reg, rs1: Reg, imm: i64 },

    // Type U load upper immediate
    Lui { rd: Reg, imm: i64 },
    Auipc { rd: Reg, imm: i64 },

    // System instructions
    Ecall,
    Ebreak,
    Csrrw { rd: Reg, rs1: Reg, csr: u16 },
    Csrrs { rd: Reg, rs1: Reg, csr: u16 },
    Csrrc { rd: Reg, rs1: Reg, csr: u16 },
    Csrrwi { rd: Reg, uimm: u8, csr: u16 },
    Csrrsi { rd: Reg, uimm: u8, csr: u16 },
    Csrrci { rd: Reg, uimm: u8, csr: u16 },
    Sret,
    Mret,
    Wfi,
    SfenceVma { rs1: Reg, rs2: Reg },

    Fence { pred: u8, succ: u8, fm: u8 },
    FenceI,

    // Floating point f signle precision
    // floating point loads
    Flw { rd: FReg, rs1: Reg, imm: i64 },
    Fld { rd: FReg, rs1: Reg, imm: i64 },
    // floating point stores
    Fsw { rs1: Reg, rs2: FReg, imm: i64 },
    Fsd { rs1: Reg, rs2: FReg, imm: i64 },
    FmvWX { rd: FReg, rs1: Reg },
    FmvXW { rd: Reg, rs1: FReg },
    FaddS { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FsubS { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FmulS { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FdivS { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FsqrtS { rd: FReg, rs1: FReg, rm: u8 },
    FsgnjS { rd: FReg, rs1: FReg, rs2: FReg },
    FsgnjnS { rd: FReg, rs1: FReg, rs2: FReg },
    FsgnjxS { rd: FReg, rs1: FReg, rs2: FReg },
    FminS { rd: FReg, rs1: FReg, rs2: FReg },
    FmaxS { rd: FReg, rs1: FReg, rs2: FReg },
    FeqS { rd: Reg, rs1: FReg, rs2: FReg },
    FltS { rd: Reg, rs1: FReg, rs2: FReg },
    FleS { rd: Reg, rs1: FReg, rs2: FReg },
    FcvtWS { rd: Reg, rs1: FReg, rm: u8 },
    FcvtWUS { rd: Reg, rs1: FReg, rm: u8 },
    FcvtLS { rd: Reg, rs1: FReg, rm: u8 },
    FcvtLUS { rd: Reg, rs1: FReg, rm: u8 },
    FcvtSW { rd: FReg, rs1: Reg, rm: u8 },
    FcvtSWU { rd: FReg, rs1: Reg, rm: u8 },
    FcvtSL { rd: FReg, rs1: Reg, rm: u8 },
    FcvtSLU { rd: FReg, rs1: Reg, rm: u8 },
    FclassS { rd: Reg, rs1: FReg },
    FmaddS { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FmsubS { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FnmsubS { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FnmaddS { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    // D extension
    FaddD { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FsubD { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FmulD { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FdivD { rd: FReg, rs1: FReg, rs2: FReg, rm: u8 },
    FsqrtD { rd: FReg, rs1: FReg, rm: u8 },
    FsgnjD { rd: FReg, rs1: FReg, rs2: FReg },
    FsgnjnD { rd: FReg, rs1: FReg, rs2: FReg },
    FsgnjxD { rd: FReg, rs1: FReg, rs2: FReg },
    FminD { rd: FReg, rs1: FReg, rs2: FReg },
    FmaxD { rd: FReg, rs1: FReg, rs2: FReg },
    FeqD { rd: Reg, rs1: FReg, rs2: FReg },
    FltD { rd: Reg, rs1: FReg, rs2: FReg },
    FleD { rd: Reg, rs1: FReg, rs2: FReg },
    FcvtWD { rd: Reg, rs1: FReg, rm: u8 },
    FcvtWUD { rd: Reg, rs1: FReg, rm: u8 },
    FcvtLD { rd: Reg, rs1: FReg, rm: u8 },
    FcvtLUD { rd: Reg, rs1: FReg, rm: u8 },
    FcvtDW { rd: FReg, rs1: Reg, rm: u8 },
    FcvtDWU { rd: FReg, rs1: Reg, rm: u8 },
    FcvtDL { rd: FReg, rs1: Reg, rm: u8 },
    FcvtDLU { rd: FReg, rs1: Reg, rm: u8 },
    FcvtDS { rd: FReg, rs1: FReg, rm: u8 },
    FcvtSD { rd: FReg, rs1: FReg, rm: u8 },
    FmvXD { rd: Reg, rs1: FReg },
    FclassD { rd: Reg, rs1: FReg },
    FmvDX { rd: FReg, rs1: Reg },
    FmaddD { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FmsubD { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FnmsubD { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    FnmaddD { rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8 },
    Undefined { raw: u32 },
}
