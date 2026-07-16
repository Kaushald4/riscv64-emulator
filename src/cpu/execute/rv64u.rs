use crate::{
    cpu::{Cpu, ExecFlow, ExecResult, execute::helper::addr, register::Reg},
    mmu::Mmu,
};

pub fn sb(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    Mmu::write8(cpu, addr, cpu.regs.read(rs2) as u8)?;

    Ok(ExecFlow::Next)
}

pub fn sh(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2) as u16;
    Mmu::write16(cpu, addr, value)?;

    Ok(ExecFlow::Next)
}

pub fn sw(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2) as u32;
    Mmu::write32(cpu, addr, value)?;

    Ok(ExecFlow::Next)
}

pub fn sd(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = cpu.regs.read(rs2);
    Mmu::write64(cpu, addr, value)?;

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
