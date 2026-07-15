use crate::{
    cpu::register::Reg,
    decode::c_formats::{
        ca_bit12, ca_funct2, ca_rd_rs1, ca_rs2, cb_funct2, cb_imm, cb_offset, cb_rs1, cb_shamt, ci_addi16sp_imm, ci_frd, ci_imm, ci_ldsp_offset, ci_lui_imm, ci_lwsp_offset, ci_rd, ci_shamt, ciw_nzuimm, ciw_rd, cj_offset, cl_frd, cl_ld_offset, cl_lw_offset, cl_rd, cl_rs1, cr_rd_rs1, cr_rs2, cs_frs2, cs_rs1, cs_rs2, cs_sd_offset, cs_sw_offset,
        css_frs2, css_rs2, css_sdsp_offset, css_swsp_offset, funct3c, quadrant,
    },
    instruction::Instruction,
};

pub fn decode_compressed(raw: u16) -> Instruction {
    match quadrant(raw) {
        0b00 => decode_q0(raw),
        0b01 => decode_q1(raw),
        0b10 => decode_q2(raw),
        _ => unreachable!(),
    }
}

fn decode_q0(raw: u16) -> Instruction {
    match funct3c(raw) {
        0b000 => decode_c_addi4spn(raw),
        0b001 => decode_c_fld(raw),
        0b010 => decode_c_lw(raw),
        0b011 => decode_c_ld(raw),
        0b101 => decode_c_fsd(raw),
        0b110 => decode_c_sw(raw),
        0b111 => decode_c_sd(raw),

        _ => Instruction::Undefined { raw: raw as u32 },
    }
}

fn decode_q1(raw: u16) -> Instruction {
    match funct3c(raw) {
        0b000 => decode_c_addi(raw),
        0b001 => decode_c_addiw(raw),
        0b010 => decode_c_li(raw),
        0b011 => decode_c_lui_addi16sp(raw),
        0b100 => decode_c_alu(raw),
        0b101 => decode_c_j(raw),
        0b110 => decode_c_beqz(raw),
        0b111 => decode_c_bnez(raw),

        _ => Instruction::Undefined { raw: raw as u32 },
    }
}

fn decode_q2(raw: u16) -> Instruction {
    match funct3c(raw) {
        0b000 => decode_c_slli(raw),
        0b001 => decode_c_fldsp(raw),
        0b010 => decode_c_lwsp(raw),
        0b011 => decode_c_ldsp(raw),
        0b100 => decode_c_jr_mv_add(raw),
        0b101 => decode_c_fsdsp(raw),
        0b110 => decode_c_swsp(raw),
        0b111 => decode_c_sdsp(raw),

        _ => Instruction::Undefined { raw: raw as u32 },
    }
}

// actual decoder functions
fn decode_c_lw(raw: u16) -> Instruction {
    Instruction::Lw { rd: cl_rd(raw), rs1: cl_rs1(raw), imm: cl_lw_offset(raw) as i64 }
}
fn decode_c_ld(raw: u16) -> Instruction {
    Instruction::Ld { rd: cl_rd(raw), rs1: cl_rs1(raw), imm: cl_ld_offset(raw) as i64 }
}
fn decode_c_sw(raw: u16) -> Instruction {
    Instruction::Sw {
        rs1: cs_rs1(raw),
        rs2: cs_rs2(raw),
        imm: cs_sw_offset(raw) as i64,
    }
}
fn decode_c_sd(raw: u16) -> Instruction {
    Instruction::Sd {
        rs1: cs_rs1(raw),
        rs2: cs_rs2(raw),
        imm: cs_sd_offset(raw) as i64,
    }
}
fn decode_c_addi4spn(raw: u16) -> Instruction {
    Instruction::Addi { rd: ciw_rd(raw), rs1: Reg::new(2), imm: ciw_nzuimm(raw) as i64 }
}
fn decode_c_fld(raw: u16) -> Instruction {
    Instruction::Fld {
        rd: cl_frd(raw),
        rs1: cl_rs1(raw),
        imm: cl_ld_offset(raw) as i64,
    }
}
fn decode_c_fsd(raw: u16) -> Instruction {
    Instruction::Fsd {
        rs1: cs_rs1(raw),
        rs2: cs_frs2(raw),
        imm: cs_sd_offset(raw) as i64,
    }
}

// Q1
fn decode_c_addi(raw: u16) -> Instruction {
    let rd = ci_rd(raw);
    let imm = ci_imm(raw);

    Instruction::Addi { rd, rs1: rd, imm }
}

fn decode_c_addiw(raw: u16) -> Instruction {
    let rd = ci_rd(raw);

    Instruction::Addiw { rd, rs1: rd, imm: ci_imm(raw) }
}

fn decode_c_li(raw: u16) -> Instruction {
    Instruction::Addi { rd: ci_rd(raw), rs1: Reg::new(0), imm: ci_imm(raw) }
}

fn decode_c_lui_addi16sp(raw: u16) -> Instruction {
    let rd = ci_rd(raw);

    if rd.idx() == 2 { Instruction::Addi { rd, rs1: rd, imm: ci_addi16sp_imm(raw) } } else { Instruction::Lui { rd, imm: ci_lui_imm(raw) } }
}

fn decode_c_j(raw: u16) -> Instruction {
    Instruction::Jal { rd: Reg::new(0), imm: cj_offset(raw) }
}

fn decode_c_beqz(raw: u16) -> Instruction {
    Instruction::Beq { rs1: cb_rs1(raw), rs2: Reg::new(0), imm: cb_offset(raw) }
}

fn decode_c_bnez(raw: u16) -> Instruction {
    Instruction::Bne { rs1: cb_rs1(raw), rs2: Reg::new(0), imm: cb_offset(raw) }
}
fn decode_c_alu(raw: u16) -> Instruction {
    match cb_funct2(raw) {
        0b00 => decode_c_srli(raw),
        0b01 => decode_c_srai(raw),
        0b10 => decode_c_andi(raw),
        0b11 => decode_c_arithmetic(raw),
        _ => unreachable!(),
    }
}

fn decode_c_srli(raw: u16) -> Instruction {
    let rd = cb_rs1(raw);
    Instruction::Srli { rd, rs1: rd, shamt: cb_shamt(raw) }
}

fn decode_c_srai(raw: u16) -> Instruction {
    let rd = cb_rs1(raw);
    Instruction::Srai { rd, rs1: rd, shamt: cb_shamt(raw) }
}

fn decode_c_andi(raw: u16) -> Instruction {
    let rd = cb_rs1(raw);
    Instruction::Andi { rd, rs1: rd, imm: cb_imm(raw) }
}

fn decode_c_arithmetic(raw: u16) -> Instruction {
    if ca_bit12(raw) { decode_c_rv64_arithmetic(raw) } else { decode_c_rv32_arithmetic(raw) }
}

fn decode_c_rv32_arithmetic(raw: u16) -> Instruction {
    let rd = ca_rd_rs1(raw);
    let rs2 = ca_rs2(raw);

    match ca_funct2(raw) {
        0b00 => Instruction::Sub { rd, rs1: rd, rs2 },
        0b01 => Instruction::Xor { rd, rs1: rd, rs2 },
        0b10 => Instruction::Or { rd, rs1: rd, rs2 },
        0b11 => Instruction::And { rd, rs1: rd, rs2 },
        _ => unreachable!(),
    }
}

fn decode_c_rv64_arithmetic(raw: u16) -> Instruction {
    let rd = ca_rd_rs1(raw);
    let rs2 = ca_rs2(raw);

    match ca_funct2(raw) {
        0b00 => Instruction::Subw { rd, rs1: rd, rs2 },
        0b01 => Instruction::Addw { rd, rs1: rd, rs2 },
        0b10 => Instruction::Undefined { raw: raw as u32 },
        0b11 => Instruction::Undefined { raw: raw as u32 },
        _ => unreachable!(),
    }
}

// Q2
fn decode_c_slli(raw: u16) -> Instruction {
    let rd = ci_rd(raw);
    Instruction::Slli { rd, rs1: rd, shamt: ci_shamt(raw) }
}

fn decode_c_fldsp(raw: u16) -> Instruction {
    Instruction::Fld {
        rd: ci_frd(raw),
        rs1: Reg::new(2),
        imm: ci_ldsp_offset(raw) as i64,
    }
}

fn decode_c_lwsp(raw: u16) -> Instruction {
    let rd = ci_rd(raw);
    Instruction::Lw { rd, rs1: Reg::new(2), imm: ci_lwsp_offset(raw) as i64 }
}

fn decode_c_ldsp(raw: u16) -> Instruction {
    let rd = ci_rd(raw);
    Instruction::Ld { rd, rs1: Reg::new(2), imm: ci_ldsp_offset(raw) as i64 }
}

fn decode_c_fsdsp(raw: u16) -> Instruction {
    Instruction::Fsd {
        rs1: Reg::new(2),
        rs2: css_frs2(raw),
        imm: css_sdsp_offset(raw) as i64,
    }
}

fn decode_c_swsp(raw: u16) -> Instruction {
    Instruction::Sw {
        rs1: Reg::new(2),
        rs2: css_rs2(raw),
        imm: css_swsp_offset(raw) as i64,
    }
}

fn decode_c_sdsp(raw: u16) -> Instruction {
    Instruction::Sd {
        rs1: Reg::new(2),
        rs2: css_rs2(raw),
        imm: css_sdsp_offset(raw) as i64,
    }
}

fn decode_c_jr_mv_add(raw: u16) -> Instruction {
    let rd = cr_rd_rs1(raw);
    let rs2 = cr_rs2(raw);
    let bit12 = ((raw >> 12) & 1) != 0;

    if !bit12 {
        if rs2.is_zero() {
            // C.JR
            Instruction::Jalr { rd: Reg::new(0), rs1: rd, imm: 0 }
        } else {
            // C.MV
            Instruction::Add { rd, rs1: Reg::new(0), rs2 }
        }
    } else {
        if rd.is_zero() && rs2.is_zero() {
            // C.EBREAK
            Instruction::Ebreak
        } else if rs2.is_zero() {
            // C.JALR
            Instruction::Jalr { rd: Reg::new(1), rs1: rd, imm: 0 }
        } else {
            // C.ADD
            Instruction::Add { rd, rs1: rd, rs2 }
        }
    }
}
