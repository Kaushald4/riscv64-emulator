use crate::cpu::Cpu;
use crate::cpu::register::Reg;

pub fn addi(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) {
    let a = cpu.regs.read(rs1) as i64;
    let result = a.wrapping_add(imm);
    cpu.regs.write(rd, result as u64);
}
