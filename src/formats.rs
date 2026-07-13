use crate::register::Reg;

#[inline]
pub const fn opcode(raw: u32) -> u32 {
    raw & 0b1111111
}

#[inline]
pub const fn rd(raw: u32) -> Reg {
    Reg::new(raw >> 7)
}

#[inline]
pub const fn funct3(raw: u32) -> u32 {
    (raw >> 12) & 0b111
}

#[inline]
pub const fn funct7(raw: u32) -> u32 {
    (raw >> 25) & 0b1111111
}

#[inline]
pub const fn rs1(raw: u32) -> Reg {
    Reg::new(raw >> 15)
}

pub const fn rs2(raw: u32) -> Reg {
    Reg::new(raw >> 20)
}

#[inline]
pub const fn imm_b(raw: u32) -> i64 {
    let imm12 = (raw >> 31) & 0b1;
    let imm11 = (raw >> 7) & 0b1;
    let imm10_5 = (raw >> 25) & 0b111111;
    let imm4_1 = (raw >> 8) & 0b1111;

    let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);

    sign_extend(imm, 13)
}

#[inline]
pub const fn imm_j(raw: u32) -> i64 {
    let imm20 = (raw >> 31) & 0b1;
    let imm19_12 = (raw >> 12) & 0b11111111;
    let imm11 = (raw >> 20) & 0b1;
    let imm10_1 = (raw >> 21) & 0b1111111111;

    let imm = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);

    sign_extend(imm, 21)
}

#[inline]
pub const fn imm_u(raw: u32) -> i64 {
    sign_extend(raw & 0xfffff000, 32)
}

// for immeidate
#[inline]
pub const fn imm_i(raw: u32) -> i64 {
    sign_extend((raw >> 20) & 0b111111111111, 12)
}

#[inline]
pub const fn shamt5(raw: u32) -> u32 {
    (raw >> 20) & 0b11111
}

#[inline]
pub const fn shamt6(raw: u32) -> u32 {
    (raw >> 20) & 0b111111
}

// atomics
pub const fn funct5(raw: u32) -> u32 {
    (raw >> 27) & 0x1f
}

#[inline]
pub const fn sign_extend(value: u32, bits: u32) -> i64 {
    let shift = 64 - bits;
    ((value as i64) << shift) >> shift
}
