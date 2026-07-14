use crate::cpu::{Cpu, register::Reg};

#[inline]
pub fn addr(cpu: &Cpu, rs1: Reg, imm: i64) -> u64 {
    cpu.regs.read(rs1).wrapping_add_signed(imm)
}
