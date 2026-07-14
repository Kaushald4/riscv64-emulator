use crate::{f_register::FReg, register::Reg};

// Common
#[inline]
pub const fn quadrant(raw: u16) -> u16 {
    raw & 0b11
}

#[inline]
pub const fn funct3c(raw: u16) -> u16 {
    (raw >> 13) & 0b111
}

/// expand compressed register encoding.
/// 000 -> x8
/// 111 -> x15
#[inline]
pub const fn creg(bits: u32) -> Reg {
    Reg::new((bits & 0b111) + 8)
}
#[inline]
pub const fn cfreg(bits: u32) -> FReg {
    FReg::new((bits & 0b111) + 8)
}
#[inline]
pub const fn cl_frd(raw: u16) -> FReg {
    cfreg((raw as u32) >> 2)
}

#[inline]
pub const fn cs_frs2(raw: u16) -> FReg {
    cfreg((raw as u32) >> 2)
}

#[inline]
pub const fn ci_frd(raw: u16) -> FReg {
    FReg::new((raw as u32) >> 7)
}

#[inline]
pub const fn css_frs2(raw: u16) -> FReg {
    FReg::new((raw as u32) >> 2)
}

// CIW Format (C.ADDI4SPN)
#[inline]
pub const fn ciw_rd(raw: u16) -> Reg {
    creg((raw as u32) >> 2)
}

#[inline]
pub const fn ciw_nzuimm(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[10:7] -> imm[9:6]
    let imm_9_6 = ((raw >> 7) & 0b1111) << 6;

    // inst[12:11] -> imm[5:4]
    let imm_5_4 = ((raw >> 11) & 0b11) << 4;

    // inst[5] -> imm[3]
    let imm_3 = ((raw >> 5) & 1) << 3;

    // inst[6] -> imm[2]
    let imm_2 = ((raw >> 6) & 1) << 2;

    imm_9_6 | imm_5_4 | imm_3 | imm_2
}

// CL Format
#[inline]
pub const fn cl_rd(raw: u16) -> Reg {
    creg((raw as u32) >> 2)
}

#[inline]
pub const fn cl_rs1(raw: u16) -> Reg {
    creg((raw as u32) >> 7)
}

#[inline]
pub const fn cl_lw_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[5] -> offset[6]
    let imm_6 = ((raw >> 5) & 1) << 6;

    // inst[12:10] -> offset[5:3]
    let imm_5_3 = ((raw >> 10) & 0b111) << 3;

    // inst[6] -> offset[2]
    let imm_2 = ((raw >> 6) & 1) << 2;

    imm_6 | imm_5_3 | imm_2
}

#[inline]
pub const fn cl_ld_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[6:5] -> offset[8:6]
    let imm_8_6 = ((raw >> 5) & 0b11) << 6;

    // inst[12:10] -> offset[5:3]
    let imm_5_3 = ((raw >> 10) & 0b111) << 3;

    imm_8_6 | imm_5_3
}

// CS Format
#[inline]
pub const fn cs_rs2(raw: u16) -> Reg {
    creg((raw as u32) >> 2)
}

#[inline]
pub const fn cs_rs1(raw: u16) -> Reg {
    creg((raw as u32) >> 7)
}

#[inline]
pub const fn cs_sw_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[5] -> offset[6]
    let imm_6 = ((raw >> 5) & 1) << 6;

    // inst[12:10] -> offset[5:3]
    let imm_5_3 = ((raw >> 10) & 0b111) << 3;

    // inst[6] -> offset[2]
    let imm_2 = ((raw >> 6) & 1) << 2;

    imm_6 | imm_5_3 | imm_2
}

#[inline]
pub const fn cs_sd_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[6:5] -> offset[8:6]
    let imm_8_6 = ((raw >> 5) & 0b11) << 6;

    // inst[12:10] -> offset[5:3]
    let imm_5_3 = ((raw >> 10) & 0b111) << 3;

    imm_8_6 | imm_5_3
}

// CR Format
#[inline]
pub const fn cr_rd_rs1(raw: u16) -> Reg {
    Reg::new((raw as u32) >> 7)
}

#[inline]
pub const fn cr_rs2(raw: u16) -> Reg {
    Reg::new((raw as u32) >> 2)
}

#[inline]
pub const fn cr_funct4(raw: u16) -> u32 {
    ((raw as u32) >> 12) & 0b1111
}

// CI Format
#[inline]
pub const fn ci_rd(raw: u16) -> Reg {
    Reg::new((raw as u32) >> 7)
}

#[inline]
pub const fn ci_rs1(raw: u16) -> Reg {
    ci_rd(raw)
}

#[inline]
pub const fn ci_lwsp_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[3:2] -> offset[7:6]
    let imm_7_6 = ((raw >> 2) & 0b11) << 6;

    // inst[12] -> offset[5]
    let imm_5 = ((raw >> 12) & 1) << 5;

    // inst[6:4] -> offset[4:2]
    let imm_4_2 = ((raw >> 4) & 0b111) << 2;

    imm_7_6 | imm_5 | imm_4_2
}

#[inline]
pub const fn ci_ldsp_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    // inst[4:2] -> offset[8:6]
    let imm_8_6 = ((raw >> 2) & 0b111) << 6;

    // inst[12] -> offset[5]
    let imm_5 = ((raw >> 12) & 1) << 5;

    // inst[6:5] -> offset[4:3]
    let imm_4_3 = ((raw >> 5) & 0b11) << 3;

    imm_8_6 | imm_5 | imm_4_3
}

/// CI immediate
/// imm[5]   <- inst[12]
/// imm[4:0] <- inst[6:2]
#[inline]
pub const fn ci_imm(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_5 = ((raw >> 12) & 1) << 5;
    let imm_4_0 = (raw >> 2) & 0b11111;

    let imm = imm_5 | imm_4_0;

    ((imm as i32) << 26 >> 26) as i64
}

/// Shift amount used by
/// C.SLLI
/// C.SRLI
/// C.SRAI
#[inline]
pub const fn ci_shamt(raw: u16) -> u32 {
    let raw = raw as u32;

    let shamt_5 = ((raw >> 12) & 1) << 5;
    let shamt_4_0 = (raw >> 2) & 0b11111;

    shamt_5 | shamt_4_0
}

/// C.ADDI16SP immediate
/// nzimm[9]    <- inst[12]
/// nzimm[4]    <- inst[6]
/// nzimm[6]    <- inst[5]
/// nzimm[8:7]  <- inst[4:3]
/// nzimm[5]    <- inst[2]
/// bit0..3 are always zero.
#[inline]
pub const fn ci_addi16sp_imm(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_9 = ((raw >> 12) & 1) << 9;
    let imm_8_7 = ((raw >> 3) & 0b11) << 7;
    let imm_6 = ((raw >> 5) & 1) << 6;
    let imm_5 = ((raw >> 2) & 1) << 5;
    let imm_4 = ((raw >> 6) & 1) << 4;

    let imm = imm_9 | imm_8_7 | imm_6 | imm_5 | imm_4;

    ((imm as i32) << 22 >> 22) as i64
}

/// Used only by C.LUI.
/// imm[17] <- inst[12]
/// imm[16:12] <- inst[6:2]
#[inline]
pub const fn ci_lui_imm(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_17 = ((raw >> 12) & 1) << 17;
    let imm_16_12 = ((raw >> 2) & 0b11111) << 12;

    let imm = imm_17 | imm_16_12;

    ((imm as i32) << 14 >> 14) as i64
}

// CSS Format
#[inline]
pub const fn css_rs2(raw: u16) -> Reg {
    Reg::new((raw as u32) >> 2)
}

/// C.SWSP offset
/// offset[7:6] <- inst[8:7]
/// offset[5:2] <- inst[12:9]
#[inline]
pub const fn css_swsp_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    let imm_7_6 = ((raw >> 7) & 0b11) << 6;
    let imm_5_2 = ((raw >> 9) & 0b1111) << 2;

    imm_7_6 | imm_5_2
}

/// C.SDSP offset
/// offset[8:6] <- inst[9:7]
/// offset[5:3] <- inst[12:10]
#[inline]
pub const fn css_sdsp_offset(raw: u16) -> u32 {
    let raw = raw as u32;

    let imm_8_6 = ((raw >> 7) & 0b111) << 6;
    let imm_5_3 = ((raw >> 10) & 0b111) << 3;

    imm_8_6 | imm_5_3
}

// CA Format
#[inline]
pub const fn ca_rd_rs1(raw: u16) -> Reg {
    creg((raw as u32) >> 7)
}

#[inline]
pub const fn ca_rs2(raw: u16) -> Reg {
    creg((raw as u32) >> 2)
}

/// funct2 (bits 6:5)
#[inline]
pub const fn ca_funct2(raw: u16) -> u32 {
    ((raw as u32) >> 5) & 0b11
}

/// funct6 (bits 15:10)
#[inline]
pub const fn ca_funct6(raw: u16) -> u32 {
    ((raw as u32) >> 10) & 0b111111
}

/// bit12 is used to distinguish
/// SUB/ADDW and friends.
#[inline]
pub const fn ca_bit12(raw: u16) -> bool {
    ((raw >> 12) & 1) != 0
}

// CB Format
#[inline]
pub const fn cb_funct2(raw: u16) -> u32 {
    ((raw as u32) >> 10) & 0b11
}

#[inline]
pub const fn cb_rs1(raw: u16) -> Reg {
    creg((raw as u32) >> 7)
}

/// Immediate used by
/// C.SRLI
/// C.SRAI
/// C.ANDI
#[inline]
pub const fn cb_imm(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_5 = ((raw >> 12) & 1) << 5;
    let imm_4_0 = (raw >> 2) & 0b11111;

    let imm = imm_5 | imm_4_0;

    ((imm as i32) << 26 >> 26) as i64
}

#[inline]
pub const fn cb_shamt(raw: u16) -> u32 {
    let raw = raw as u32;

    let shamt_5 = ((raw >> 12) & 1) << 5;
    let shamt_4_0 = (raw >> 2) & 0b11111;

    shamt_5 | shamt_4_0
}

/// Branch offset.
/// offset[8]   <- inst[12]
/// offset[7:6] <- inst[6:5]
/// offset[5]   <- inst[2]
/// offset[4:3] <- inst[11:10]
/// offset[2:1] <- inst[4:3]
/// bit0 = 0
#[inline]
pub const fn cb_offset(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_8 = ((raw >> 12) & 1) << 8;
    let imm_7_6 = ((raw >> 5) & 0b11) << 6;
    let imm_5 = ((raw >> 2) & 1) << 5;
    let imm_4_3 = ((raw >> 10) & 0b11) << 3;
    let imm_2_1 = ((raw >> 3) & 0b11) << 1;

    let imm = imm_8 | imm_7_6 | imm_5 | imm_4_3 | imm_2_1;

    ((imm as i32) << 23 >> 23) as i64
}

// CJ Format
/// Jump offset.
/// offset[11]   <- inst[12]
/// offset[10]   <- inst[8]
/// offset[9:8]  <- inst[10:9]
/// offset[7]    <- inst[6]
/// offset[6]    <- inst[7]
/// offset[5]    <- inst[2]
/// offset[4]    <- inst[11]
/// offset[3:1]  <- inst[5:3]
/// bit0 = 0
#[inline]
pub const fn cj_offset(raw: u16) -> i64 {
    let raw = raw as u32;

    let imm_11 = ((raw >> 12) & 1) << 11;
    let imm_10 = ((raw >> 8) & 1) << 10;
    let imm_9_8 = ((raw >> 9) & 0b11) << 8;
    let imm_7 = ((raw >> 6) & 1) << 7;
    let imm_6 = ((raw >> 7) & 1) << 6;
    let imm_5 = ((raw >> 2) & 1) << 5;
    let imm_4 = ((raw >> 11) & 1) << 4;
    let imm_3_1 = ((raw >> 3) & 0b111) << 1;

    let imm = imm_11 | imm_10 | imm_9_8 | imm_7 | imm_6 | imm_5 | imm_4 | imm_3_1;

    ((imm as i32) << 20 >> 20) as i64
}
