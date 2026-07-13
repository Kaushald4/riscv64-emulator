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

// for immeidate
#[inline]
pub const fn imm_i(raw: u32) -> i64 {
    ((raw as i32) >> 20) as i64
}

#[inline]
pub const fn shamt(raw: u32) -> i64 {
    (((raw as i32) >> 20) & 0b11111) as i64
}

// atomics
pub const fn funct5(raw: u32) -> u32 {
    (raw >> 27) & 0x1f
}
