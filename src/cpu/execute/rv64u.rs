use crate::{
    cpu::{Cpu, ExecFlow, ExecResult, register::Reg},
    trap::Trap,
};

#[inline]
fn effective_address(cpu: &Cpu, rs1: Reg, imm: i64) -> u64 {
    cpu.regs.read(rs1).wrapping_add_signed(imm)
}

pub fn sb(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = effective_address(cpu, rs1, imm);

    cpu.bus.write8(addr, cpu.regs.read(rs2) as u8)?;

    Ok(ExecFlow::Next)
}

pub fn sh(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = effective_address(cpu, rs1, imm);

    if addr & 1 != 0 {
        return Err(Trap::StoreAddressMisaligned);
    }

    cpu.bus.write16(addr, cpu.regs.read(rs2) as u16)?;

    Ok(ExecFlow::Next)
}

pub fn sw(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = effective_address(cpu, rs1, imm);

    if addr & 3 != 0 {
        return Err(Trap::StoreAddressMisaligned);
    }

    cpu.bus.write32(addr, cpu.regs.read(rs2) as u32)?;

    Ok(ExecFlow::Next)
}

pub fn sd(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    let addr = effective_address(cpu, rs1, imm);

    if addr & 7 != 0 {
        return Err(Trap::StoreAddressMisaligned);
    }

    cpu.bus.write64(addr, cpu.regs.read(rs2))?;

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
