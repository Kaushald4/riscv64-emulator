use crate::{
    formats::{funct3, funct7, imm_i, opcode, rd, rs1, rs2, shamt},
    instruction::Instruction,
    opcode::{OP, OP_IMM, OP_IMM_W},
};

pub fn decode(raw: u32) -> Instruction {
    match opcode(raw) {
        OP_IMM => decode_op_imm(raw),
        OP_IMM_W => decode_op_w(raw),
        OP => decode_op(raw),
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_imm(raw: u32) -> Instruction {
    match funct3(raw) {
        0b000 => Instruction::Addi { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b010 => Instruction::Slti { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b011 => Instruction::Sltiu { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b100 => Instruction::Xori { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b110 => Instruction::Ori { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b111 => Instruction::Andi { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b001 => Instruction::Slli { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
        0b101 => match funct7(raw) {
            0b0000000 => Instruction::Srli { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
            0b0100000 => Instruction::Srai { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_w(raw: u32) -> Instruction {
    match funct3(raw) {
        0b000 => Instruction::Addiw { rd: rd(raw), rs1: rs1(raw), imm: imm_i(raw) },
        0b001 => Instruction::Slliw { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
        0b101 => match funct7(raw) {
            0b0000000 => Instruction::Srliw { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
            0b0100000 => Instruction::Sraiw { rd: rd(raw), rs1: rs1(raw), shamt: shamt(raw) },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op(raw: u32) -> Instruction {
    match (funct3(raw), funct7(raw)) {
        (0b0000000, 0b000) => Instruction::Add { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0100000, 0b000) => Instruction::Sub { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b001) => Instruction::Sll { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b010) => Instruction::Slt { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b011) => Instruction::Sltu { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b100) => Instruction::Xor { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b101) => Instruction::Srl { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0100000, 0b101) => Instruction::Sra { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b110) => Instruction::Or { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000000, 0b111) => Instruction::And { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        // RV32M standard extension
        (0b0000001, 0b000) => Instruction::Mul { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b001) => Instruction::Mulh { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b010) => Instruction::Mulhsu { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b011) => Instruction::Mulhu { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b100) => Instruction::Div { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b101) => Instruction::Divu { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b110) => Instruction::Rem { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },
        (0b0000001, 0b111) => Instruction::Remu { rd: rd(raw), rs1: rs1(raw), rs2: rs2(raw) },

        _ => Instruction::Undefined { raw },
    }
}
