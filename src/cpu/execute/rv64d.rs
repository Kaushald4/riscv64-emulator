use crate::cpu::{Cpu, ExecFlow, ExecResult, execute::helper::addr, f_register::FReg, register::Reg};
// memory
pub fn fld(cpu: &mut Cpu, rd: FReg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let bits = cpu.bus.read64(addr)?;

    cpu.f_regs.write_f64_bits(rd, bits);

    Ok(ExecFlow::Next)
}

pub fn fsd(cpu: &mut Cpu, rs1: Reg, rs2: FReg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let bits = cpu.f_regs.read_f64_bits(rs2);

    cpu.bus.write64(addr, bits)?;

    Ok(ExecFlow::Next)
}
// moves
pub fn fmv_d_x(cpu: &mut Cpu, rd: FReg, rs1: Reg) -> ExecResult {
    let bits = cpu.regs.read(rs1);

    cpu.f_regs.write_f64_bits(rd, bits);

    Ok(ExecFlow::Next)
}

pub fn fmv_x_d(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let bits = cpu.f_regs.read_f64_bits(rs1);

    cpu.regs.write(rd, bits);

    Ok(ExecFlow::Next)
}
