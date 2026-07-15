use crate::{
    decode::{
        f_formats::{frd, frs1, frs2, frs3},
        formats::{funct3, imm_i, rd, rs1, rs2, sign_extend},
    },
    instruction::Instruction,
};

pub fn decode_load_fp(raw: u32) -> Instruction {
    let rd = frd(raw);
    let rs1 = rs1(raw);
    let imm = imm_i(raw);

    match funct3(raw) {
        0b010 => Instruction::Flw { rd, rs1, imm },

        0b011 => Instruction::Fld { rd: frd(raw), rs1, imm },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_store_fp(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = frs2(raw);

    let imm11_5 = (raw >> 25) & 0x7f;
    let imm4_0 = (raw >> 7) & 0x1f;
    let imm = sign_extend((imm11_5 << 5) | imm4_0, 12);

    match funct3(raw) {
        0b010 => Instruction::Fsw { rs1, rs2, imm },
        0b011 => Instruction::Fsd { rs1, rs2, imm },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_op_fp(raw: u32) -> Instruction {
    let funct7 = raw >> 25;

    match funct7 {
        0b0000000 => Instruction::FaddS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0000001 => Instruction::FaddD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },

        0b0000100 => Instruction::FsubS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0000101 => Instruction::FsubD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0001000 => Instruction::FmulS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0001001 => Instruction::FmulD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0001100 => Instruction::FdivS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0001101 => Instruction::FdivD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rm: funct3(raw) as u8,
        },
        0b0101100 => match rs2(raw).idx() {
            0 => Instruction::FsqrtS { rd: frd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            _ => Instruction::Undefined { raw },
        },
        0b0101101 => match rs2(raw).idx() {
            0 => Instruction::FsqrtD { rd: frd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },

        0b0010000 => match funct3(raw) {
            0b000 => Instruction::FsgnjS { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FsgnjnS { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b010 => Instruction::FsgnjxS { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b0010001 => match funct3(raw) {
            0b000 => Instruction::FsgnjD { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FsgnjnD { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b010 => Instruction::FsgnjxD { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b0010100 => match funct3(raw) {
            0b000 => Instruction::FminS { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FmaxS { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b0010101 => match funct3(raw) {
            0b000 => Instruction::FminD { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FmaxD { rd: frd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },

        0b1010000 => match funct3(raw) {
            0b010 => Instruction::FeqS { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FltS { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b000 => Instruction::FleS { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b1010001 => match funct3(raw) {
            0b010 => Instruction::FeqD { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b001 => Instruction::FltD { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },
            0b000 => Instruction::FleD { rd: rd(raw), rs1: frs1(raw), rs2: frs2(raw) },

            _ => Instruction::Undefined { raw },
        },

        0b1110000 => match (funct3(raw), rs2(raw).idx()) {
            (0b000, 0b00000) => Instruction::FmvXW { rd: rd(raw), rs1: frs1(raw) },

            (0b001, 0b00000) => Instruction::FclassS { rd: rd(raw), rs1: frs1(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b1100000 => match rs2(raw).idx() {
            0b00000 => Instruction::FcvtWS { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00001 => Instruction::FcvtWUS { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00010 => Instruction::FcvtLS { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00011 => Instruction::FcvtLUS { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },
        0b1100001 => match rs2(raw).idx() {
            0b00000 => Instruction::FcvtWD { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00001 => Instruction::FcvtWUD { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00010 => Instruction::FcvtLD { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },
            0b00011 => Instruction::FcvtLUD { rd: rd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },

        0b1101000 => match rs2(raw).idx() {
            0b00000 => Instruction::FcvtSW { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00001 => Instruction::FcvtSWU { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00010 => Instruction::FcvtSL { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00011 => Instruction::FcvtSLU { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },
        0b1101001 => match rs2(raw).idx() {
            0b00000 => Instruction::FcvtDW { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00001 => Instruction::FcvtDWU { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00010 => Instruction::FcvtDL { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            0b00011 => Instruction::FcvtDLU { rd: frd(raw), rs1: rs1(raw), rm: funct3(raw) as u8 },
            _ => Instruction::Undefined { raw },
        },
        0b0100001 => match rs2(raw).idx() {
            0b00000 => Instruction::FcvtDS { rd: frd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },
        0b0100000 => match rs2(raw).idx() {
            0b00001 => Instruction::FcvtSD { rd: frd(raw), rs1: frs1(raw), rm: funct3(raw) as u8 },

            _ => Instruction::Undefined { raw },
        },
        0b1110001 => match (funct3(raw), rs2(raw).idx()) {
            (0b000, 0b00000) => Instruction::FmvXD { rd: rd(raw), rs1: frs1(raw) },

            (0b001, 0b00000) => Instruction::FclassD { rd: rd(raw), rs1: frs1(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b1111001 => match (funct3(raw), rs2(raw).idx()) {
            (0b000, 0b00000) => Instruction::FmvDX { rd: frd(raw), rs1: rs1(raw) },

            _ => Instruction::Undefined { raw },
        },
        0b1111000 => match (funct3(raw), rs2(raw).idx()) {
            (0b000, 0) => Instruction::FmvWX { rd: frd(raw), rs1: rs1(raw) },

            _ => Instruction::Undefined { raw },
        },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_madd(raw: u32) -> Instruction {
    match (raw >> 25) & 0b11 {
        0b00 => Instruction::FmaddS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        0b01 => Instruction::FmaddD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_msub(raw: u32) -> Instruction {
    match (raw >> 25) & 0b11 {
        0b00 => Instruction::FmsubS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        0b01 => Instruction::FmsubD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_nmsub(raw: u32) -> Instruction {
    match (raw >> 25) & 0b11 {
        0b00 => Instruction::FnmsubS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        0b01 => Instruction::FnmsubD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        _ => Instruction::Undefined { raw },
    }
}

pub fn decode_nmadd(raw: u32) -> Instruction {
    match (raw >> 25) & 0b11 {
        0b00 => Instruction::FnmaddS {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        0b01 => Instruction::FnmaddD {
            rd: frd(raw),
            rs1: frs1(raw),
            rs2: frs2(raw),
            rs3: frs3(raw),
            rm: funct3(raw) as u8,
        },

        _ => Instruction::Undefined { raw },
    }
}
