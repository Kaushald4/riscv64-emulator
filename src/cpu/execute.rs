use crate::{
    cpu::Cpu,
    instruction::{DecodedInstruction, Instruction},
    trap::Trap,
};

pub mod rv64i;

pub fn execute(decoded: DecodedInstruction, cpu: &mut Cpu) -> Result<(), Trap> {
    match decoded.instruction {
        // ALU
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

        //Loads
        Instruction::Lb { rd, rs1, imm } => rv64i::lb(cpu, rd, rs1, imm),
        Instruction::Lbu { rd, rs1, imm } => rv64i::lbu(cpu, rd, rs1, imm),
        Instruction::Lh { rd, rs1, imm } => rv64i::lh(cpu, rd, rs1, imm),
        Instruction::Lhu { rd, rs1, imm } => rv64i::lhu(cpu, rd, rs1, imm),
        Instruction::Lw { rd, rs1, imm } => rv64i::lw(cpu, rd, rs1, imm),
        Instruction::Lwu { rd, rs1, imm } => rv64i::lwu(cpu, rd, rs1, imm),
        Instruction::Ld { rd, rs1, imm } => rv64i::ld(cpu, rd, rs1, imm),
        Instruction::Undefined { raw } => Err(Trap::IllegalInstruction(raw)),

        _ => unreachable!(),
    }
}
