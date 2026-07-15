use crate::cpu::{Cpu, ExecFlow, ExecResult, f_register::FReg, register::Reg};

// memory related
pub fn flw(cpu: &mut Cpu, rd: FReg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = cpu.regs.read(rs1).wrapping_add_signed(imm);

    let bits = cpu.bus.read32(addr)?;

    cpu.f_regs.write_f32_bits(rd, bits);

    Ok(ExecFlow::Next)
}
pub fn fsw(cpu: &mut Cpu, rs1: Reg, rs2: FReg, imm: i64) -> ExecResult {
    let addr = cpu.regs.read(rs1).wrapping_add_signed(imm);

    let bits = cpu.f_regs.read_f32_bits(rs2);

    cpu.bus.write32(addr, bits)?;

    Ok(ExecFlow::Next)
}

// moves
pub fn fmv_w_x(cpu: &mut Cpu, rd: FReg, rs1: Reg) -> ExecResult {
    let bits = cpu.regs.read(rs1) as u32;

    cpu.f_regs.write_f32_bits(rd, bits);

    Ok(ExecFlow::Next)
}
pub fn fmv_x_w(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let bits = cpu.f_regs.read_f32_raw_bits(rs1);
    cpu.regs.write(rd, bits as i32 as i64 as u64);

    Ok(ExecFlow::Next)
}
