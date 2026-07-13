use crate::{
    formats::{funct3, funct5, funct7, imm_i, opcode, rd, rs1, rs2, shamt},
    instruction::Instruction,
    opcode::{OP, OP_ATOMIC, OP_IMM, OP_IMM_W, OP_W},
};

pub fn decode(raw: u32) -> Instruction {
    match opcode(raw) {
        OP_IMM => decode_op_imm(raw),
        OP_IMM_W => decode_op_imm_w(raw),
        OP => decode_op(raw),
        OP_W => decode_op_w(raw),
        OP_ATOMIC => decode_atomic(raw),
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

fn decode_op_imm_w(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rd = rd(raw);
    let imm = imm_i(raw);
    let shamt = shamt(raw);

    match funct3(raw) {
        0b000 => Instruction::Addiw { rd, rs1, imm },
        0b001 => Instruction::Slliw { rd, rs1, shamt },
        0b101 => match funct7(raw) {
            0b0000000 => Instruction::Srliw { rd, rs1, shamt },
            0b0100000 => Instruction::Sraiw { rd, rs1, shamt },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct3(raw), funct7(raw)) {
        (0b0000000, 0b000) => Instruction::Add { rd, rs1, rs2 },
        (0b0100000, 0b000) => Instruction::Sub { rd, rs1, rs2 },
        (0b0000000, 0b001) => Instruction::Sll { rd, rs1, rs2 },
        (0b0000000, 0b010) => Instruction::Slt { rd, rs1, rs2 },
        (0b0000000, 0b011) => Instruction::Sltu { rd, rs1, rs2 },
        (0b0000000, 0b100) => Instruction::Xor { rd, rs1, rs2 },
        (0b0000000, 0b101) => Instruction::Srl { rd, rs1, rs2 },
        (0b0100000, 0b101) => Instruction::Sra { rd, rs1, rs2 },
        (0b0000000, 0b110) => Instruction::Or { rd, rs1, rs2 },
        (0b0000000, 0b111) => Instruction::And { rd, rs1, rs2 },
        // RV32M standard extension
        (0b0000001, 0b000) => Instruction::Mul { rd, rs1, rs2 },
        (0b0000001, 0b001) => Instruction::Mulh { rd, rs1, rs2 },
        (0b0000001, 0b010) => Instruction::Mulhsu { rd, rs1, rs2 },
        (0b0000001, 0b011) => Instruction::Mulhu { rd, rs1, rs2 },
        (0b0000001, 0b100) => Instruction::Div { rd, rs1, rs2 },
        (0b0000001, 0b101) => Instruction::Divu { rd, rs1, rs2 },
        (0b0000001, 0b110) => Instruction::Rem { rd, rs1, rs2 },
        (0b0000001, 0b111) => Instruction::Remu { rd, rs1, rs2 },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_w(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct3(raw), funct7(raw)) {
        (0b0000000, 0b000) => Instruction::Addw { rd, rs1, rs2 },
        (0b0100000, 0b000) => Instruction::Subw { rd, rs1, rs2 },
        (0b0000000, 0b001) => Instruction::Sllw { rd, rs1, rs2 },
        (0b0000000, 0b101) => Instruction::Srlw { rd, rs1, rs2 },
        (0b0100000, 0b101) => Instruction::Sraw { rd, rs1, rs2 },
        // RV64M word multiply/divide instruction
        (0b0000001, 0b000) => Instruction::Mulw { rd, rs1, rs2 },
        (0b0000001, 0b100) => Instruction::Divw { rd, rs1, rs2 },
        (0b0000001, 0b101) => Instruction::Divuw { rd, rs1, rs2 },
        (0b0000001, 0b110) => Instruction::Remw { rd, rs1, rs2 },
        (0b0000001, 0b111) => Instruction::Remuw { rd, rs1, rs2 },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_atomic(raw: u32) -> Instruction {
    let rl = ((raw >> 25) & 1) != 0;
    let aq = ((raw >> 26) & 1) != 0;
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct5(raw), funct3(raw)) {
        (0b00010, 0b010) => {
            if rs2.idx() != 0 {
                return Instruction::Undefined { raw };
            }
            Instruction::Lrw { rd, rs1, rl, aq }
        }
        (0b00011, 0b010) => Instruction::Scw { rd, rs1, rs2, rl, aq },
        (0b00001, 0b010) => Instruction::Amoswapw { rd, rs1, rs2, rl, aq },
        (0b00000, 0b010) => Instruction::Amoaddw { rd, rs1, rs2, rl, aq },
        (0b00100, 0b010) => Instruction::Amoxorw { rd, rs1, rs2, rl, aq },
        (0b01100, 0b010) => Instruction::Amoandw { rd, rs1, rs2, rl, aq },
        (0b01000, 0b010) => Instruction::Amoorw { rd, rs1, rs2, rl, aq },
        (0b10000, 0b010) => Instruction::Amominw { rd, rs1, rs2, rl, aq },
        (0b10100, 0b010) => Instruction::Amomaxw { rd, rs1, rs2, rl, aq },
        (0b11000, 0b010) => Instruction::Amominuw { rd, rs1, rs2, rl, aq },
        (0b11100, 0b010) => Instruction::Amomaxuw { rd, rs1, rs2, rl, aq },
        // RV64A atomic instructions
        (0b00010, 0b011) => {
            if rs2.idx() != 0 {
                return Instruction::Undefined { raw };
            }
            Instruction::Lrd { rd, rs1, rl, aq }
        }
        (0b00011, 0b011) => Instruction::Scd { rd, rs1, rs2, rl, aq },
        (0b00001, 0b011) => Instruction::Amoswapd { rd, rs1, rs2, rl, aq },
        (0b00000, 0b011) => Instruction::Amoaddd { rd, rs1, rs2, rl, aq },
        (0b00100, 0b011) => Instruction::Amoxord { rd, rs1, rs2, rl, aq },
        (0b01100, 0b011) => Instruction::Amoandd { rd, rs1, rs2, rl, aq },
        (0b01000, 0b011) => Instruction::Amoord { rd, rs1, rs2, rl, aq },
        (0b10000, 0b011) => Instruction::Amomind { rd, rs1, rs2, rl, aq },
        (0b10100, 0b011) => Instruction::Amomaxd { rd, rs1, rs2, rl, aq },
        (0b11000, 0b011) => Instruction::Amominud { rd, rs1, rs2, rl, aq },
        (0b11100, 0b011) => Instruction::Amomaxud { rd, rs1, rs2, rl, aq },

        _ => Instruction::Undefined { raw },
    }
}
