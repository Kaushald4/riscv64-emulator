use crate::cpu::Cpu;
use crate::cpu::register::Reg;
use crate::trap::Trap;

#[inline]
fn addr(cpu: &Cpu, rs1: Reg, imm: i64) -> u64 {
    cpu.regs.read(rs1).wrapping_add_signed(imm)
}

// ALU
pub fn addi(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i64;
    let result = a.wrapping_add(imm);
    cpu.regs.write(rd, result as u64);

    Ok(())
}

pub fn slti(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i64;
    cpu.regs.write(rd, (a < imm) as u64);

    Ok(())
}

pub fn sltiu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, (a < imm as u64) as u64);

    Ok(())
}

pub fn xori(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a ^ imm as u64);

    Ok(())
}

pub fn ori(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a | imm as u64);

    Ok(())
}

pub fn andi(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a & imm as u64);

    Ok(())
}

pub fn slli(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a << shamt);

    Ok(())
}

pub fn srli(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a >> shamt);

    Ok(())
}

pub fn srai(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i64;
    cpu.regs.write(rd, (a >> shamt) as u64);

    Ok(())
}

// RV64-only immediate instructions (OP IMM 32)
pub fn addiw(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i32;
    let result = a.wrapping_add(imm as i32);
    cpu.regs.write(rd, result as i64 as u64);

    Ok(())
}

pub fn slliw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i32;
    let result = a.wrapping_shl(shamt);
    cpu.regs.write(rd, result as i64 as u64);

    Ok(())
}

pub fn srliw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as u32;
    let result = a >> shamt;
    cpu.regs.write(rd, (result as i32 as i64) as u64);

    Ok(())
}

pub fn sraiw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> Result<(), Trap> {
    let a = cpu.regs.read(rs1) as i32;
    let result = a >> shamt;
    cpu.regs.write(rd, result as i64 as u64);

    Ok(())
}

// Loads
pub fn lb(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    let value = cpu.bus.read8(addr)? as i8 as i64 as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn lbu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    let value = cpu.bus.read8(addr)? as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn lh(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    if addr & 1 != 0 {
        return Err(Trap::LoadAddressMisaligned);
    }

    let value = cpu.bus.read16(addr)? as i16 as i64 as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn lhu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    let value = cpu.bus.read16(addr)? as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn lw(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    if addr & 3 != 0 {
        return Err(Trap::LoadAddressMisaligned);
    }

    let value = cpu.bus.read32(addr)? as i32 as i64 as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn lwu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    let value = cpu.bus.read32(addr)? as u64;

    cpu.regs.write(rd, value);

    Ok(())
}

pub fn ld(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> Result<(), Trap> {
    let addr = addr(cpu, rs1, imm);

    if addr & 7 != 0 {
        return Err(Trap::LoadAddressMisaligned);
    }

    let value = cpu.bus.read64(addr)?;

    cpu.regs.write(rd, value);

    Ok(())
}
