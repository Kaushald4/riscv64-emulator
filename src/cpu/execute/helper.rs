use crate::cpu::{Cpu, register::Reg};

#[inline]
pub fn addr(cpu: &Cpu, rs1: Reg, imm: i64) -> u64 {
    cpu.regs.read(rs1).wrapping_add_signed(imm)
}

#[inline]
pub fn get_bits(value: u64, shift: u64, width: u64) -> u64 {
    (value >> shift) & ((1 << width) - 1)
}

#[inline]
pub fn set_bit(value: &mut u64, bit: u64, enabled: bool) {
    if enabled {
        *value |= 1 << bit;
    } else {
        *value &= !(1 << bit);
    }
}
