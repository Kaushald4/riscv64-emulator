use crate::{
    cpu::{Cpu, ExecResult},
    instruction::{DecodedInstruction, Instruction},
    trap::Trap,
};

pub mod csr_execute;
pub mod helper;
pub mod rv64i;
pub mod rv64u;
pub mod system;

pub fn execute(decoded: DecodedInstruction, cpu: &mut Cpu) -> ExecResult {
    println!("{:?}", decoded.instruction);
    match decoded.instruction {
        // ALU Type I
        Instruction::Addi { rd, rs1, imm } => rv64i::addi(cpu, rd, rs1, imm),
        Instruction::Slti { rd, rs1, imm } => rv64i::slti(cpu, rd, rs1, imm),
        Instruction::Sltiu { rd, rs1, imm } => rv64i::sltiu(cpu, rd, rs1, imm),
        Instruction::Xori { rd, rs1, imm } => rv64i::xori(cpu, rd, rs1, imm),
        Instruction::Ori { rd, rs1, imm } => rv64i::ori(cpu, rd, rs1, imm),
        Instruction::Andi { rd, rs1, imm } => rv64i::andi(cpu, rd, rs1, imm),
        Instruction::Slli { rd, rs1, shamt } => rv64i::slli(cpu, rd, rs1, shamt),
        Instruction::Srli { rd, rs1, shamt } => rv64i::srli(cpu, rd, rs1, shamt),
        Instruction::Srai { rd, rs1, shamt } => rv64i::srai(cpu, rd, rs1, shamt),
        // RV64-only immediate instructions (OP IMM 32)
        Instruction::Addiw { rd, rs1, imm } => rv64i::addiw(cpu, rd, rs1, imm),
        Instruction::Slliw { rd, rs1, shamt } => rv64i::slliw(cpu, rd, rs1, shamt),
        Instruction::Srliw { rd, rs1, shamt } => rv64i::srliw(cpu, rd, rs1, shamt),
        Instruction::Sraiw { rd, rs1, shamt } => rv64i::sraiw(cpu, rd, rs1, shamt),
        // register to register alu
        Instruction::Add { rd, rs1, rs2 } => rv64i::add(cpu, rd, rs1, rs2),
        Instruction::Sub { rd, rs1, rs2 } => rv64i::sub(cpu, rd, rs1, rs2),
        Instruction::Sll { rd, rs1, rs2 } => rv64i::sll(cpu, rd, rs1, rs2),
        Instruction::Srl { rd, rs1, rs2 } => rv64i::srl(cpu, rd, rs1, rs2),
        Instruction::Sra { rd, rs1, rs2 } => rv64i::sra(cpu, rd, rs1, rs2),
        Instruction::Slt { rd, rs1, rs2 } => rv64i::slt(cpu, rd, rs1, rs2),
        Instruction::Sltu { rd, rs1, rs2 } => rv64i::sltu(cpu, rd, rs1, rs2),
        Instruction::Xor { rd, rs1, rs2 } => rv64i::xor(cpu, rd, rs1, rs2),
        Instruction::Or { rd, rs1, rs2 } => rv64i::or(cpu, rd, rs1, rs2),
        Instruction::And { rd, rs1, rs2 } => rv64i::and(cpu, rd, rs1, rs2),
        // OP 32
        Instruction::Addw { rd, rs1, rs2 } => rv64i::addw(cpu, rd, rs1, rs2),
        Instruction::Subw { rd, rs1, rs2 } => rv64i::subw(cpu, rd, rs1, rs2),
        Instruction::Sllw { rd, rs1, rs2 } => rv64i::sllw(cpu, rd, rs1, rs2),
        Instruction::Srlw { rd, rs1, rs2 } => rv64i::srlw(cpu, rd, rs1, rs2),
        Instruction::Sraw { rd, rs1, rs2 } => rv64i::sraw(cpu, rd, rs1, rs2),
        // Jumps
        Instruction::Jal { rd, imm } => rv64i::jal(cpu, rd, imm),
        Instruction::Jalr { rd, rs1, imm } => rv64i::jalr(cpu, rd, rs1, imm),
        // Csr
        Instruction::Csrrw { rd, rs1, csr } => csr_execute::csrrw(cpu, rd, rs1, csr),
        Instruction::Csrrs { rd, rs1, csr } => csr_execute::csrrs(cpu, rd, rs1, csr),
        Instruction::Csrrc { rd, rs1, csr } => csr_execute::csrrc(cpu, rd, rs1, csr),
        Instruction::Csrrwi { rd, uimm, csr } => csr_execute::csrrwi(cpu, rd, uimm, csr),
        Instruction::Csrrsi { rd, uimm, csr } => csr_execute::csrrsi(cpu, rd, uimm, csr),
        Instruction::Csrrci { rd, uimm, csr } => csr_execute::csrrci(cpu, rd, uimm, csr),
        // Branches
        Instruction::Bne { rs1, rs2, imm } => rv64i::bne(cpu, rs1, rs2, imm),
        Instruction::Beq { rs1, rs2, imm } => rv64i::beq(cpu, rs1, rs2, imm),
        Instruction::Blt { rs1, rs2, imm } => rv64i::blt(cpu, rs1, rs2, imm),
        Instruction::Bge { rs1, rs2, imm } => rv64i::bge(cpu, rs1, rs2, imm),
        Instruction::Bltu { rs1, rs2, imm } => rv64i::bltu(cpu, rs1, rs2, imm),
        Instruction::Bgeu { rs1, rs2, imm } => rv64i::bgeu(cpu, rs1, rs2, imm),

        //Loads Type S
        Instruction::Lb { rd, rs1, imm } => rv64i::lb(cpu, rd, rs1, imm),
        Instruction::Lbu { rd, rs1, imm } => rv64i::lbu(cpu, rd, rs1, imm),
        Instruction::Lh { rd, rs1, imm } => rv64i::lh(cpu, rd, rs1, imm),
        Instruction::Lhu { rd, rs1, imm } => rv64i::lhu(cpu, rd, rs1, imm),
        Instruction::Lw { rd, rs1, imm } => rv64i::lw(cpu, rd, rs1, imm),
        Instruction::Lwu { rd, rs1, imm } => rv64i::lwu(cpu, rd, rs1, imm),
        Instruction::Ld { rd, rs1, imm } => rv64i::ld(cpu, rd, rs1, imm),

        //Store
        Instruction::Sb { rs1, rs2, imm } => rv64u::sb(cpu, rs1, rs2, imm),
        Instruction::Sh { rs1, rs2, imm } => rv64u::sh(cpu, rs1, rs2, imm),
        Instruction::Sw { rs1, rs2, imm } => rv64u::sw(cpu, rs1, rs2, imm),
        Instruction::Sd { rs1, rs2, imm } => rv64u::sd(cpu, rs1, rs2, imm),

        Instruction::Lui { rd, imm } => rv64u::lui(cpu, rd, imm),
        Instruction::Auipc { rd, imm } => rv64u::auipc(cpu, rd, imm),

        // System
        Instruction::Fence { pred, succ, fm } => system::fence(cpu, pred, succ, fm),
        Instruction::FenceI => system::fence_i(cpu),
        Instruction::Ebreak => system::ebreak(cpu),
        Instruction::Ecall => system::ecall(cpu),

        Instruction::Undefined { raw } => Err(Trap::IllegalInstruction(raw)),

        _ => unreachable!(),
    }
}
