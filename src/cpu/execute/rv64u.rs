use crate::cpu::{Cpu, ExecFlow, ExecResult, execute::helper::addr, register::Reg};

pub fn sb(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);

    cpu.bus.write8(addr, cpu.regs.read(rs2) as u8)?;

    Ok(ExecFlow::Next)
}

pub fn sh(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2) as u16;
    cpu.bus.write16(addr, value)?;

    Ok(ExecFlow::Next)
}

pub fn sw(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2) as u32;
    cpu.bus.write32(addr, value)?;

    Ok(ExecFlow::Next)
}

pub fn sd(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2);
    cpu.bus.write64(addr, value)?;

    Ok(ExecFlow::Next)
}

pub fn lui(cpu: &mut Cpu, rd: Reg, imm: i64) -> ExecResult {
    cpu.regs.write(rd, imm as u64);

    Ok(ExecFlow::Next)
}

pub fn auipc(cpu: &mut Cpu, rd: Reg, imm: i64) -> ExecResult {
    let value = cpu.pc.wrapping_add_signed(imm);

    cpu.regs.write(rd, value);
    Ok(ExecFlow::Next)
}
