use crate::{
    formats::{funct3, funct7, imm_i, opcode, rd, rs1, shamt},
    instruction::Instruction,
    opcode::OP_IMM,
};

pub fn decode(raw: u32) -> Instruction {
    match opcode(raw) {
        OP_IMM => decode_op_imm(raw),
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_imm(raw: u32) -> Instruction {
    match funct3(raw) {
        0b000 => Instruction::Addi {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b010 => Instruction::Slti {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b011 => Instruction::Sltiu {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b100 => Instruction::Xori {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b110 => Instruction::Ori {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b111 => Instruction::Andi {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        0b001 => Instruction::Slli {
            rd: rd(raw),
            rs1: rs1(raw),
            shamt: shamt(raw),
        },
        0b101 => match funct7(raw) {
            0b0000000 => Instruction::Srli {
                rd: rd(raw),
                rs1: rs1(raw),
                shamt: shamt(raw),
            },
            0b0100000 => Instruction::Srai {
                rd: rd(raw),
                rs1: rs1(raw),
                shamt: shamt(raw),
            },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}
