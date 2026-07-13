use crate::{
    formats::{funct3, imm_i, opcode, rd, rs1},
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
        0x0 => Instruction::Addi {
            rd: rd(raw),
            rs1: rs1(raw),
            imm: imm_i(raw),
        },
        _ => Instruction::Undefined { raw },
    }
}
