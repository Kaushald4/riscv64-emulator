use crate::cpu::{Cpu, ExecFlow, ExecResult, register::Reg};

pub fn mul(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let a = cpu.regs.read(rs1);
    let b = cpu.regs.read(rs2);
    cpu.regs.write(rd, a.wrapping_mul(b));

    Ok(ExecFlow::Next)
}

pub fn mulhu(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let a = cpu.regs.read(rs1) as u128;
    let b = cpu.regs.read(rs2) as u128;
    let result = a * b;
    cpu.regs.write(rd, (result >> 64) as u64);

    Ok(ExecFlow::Next)
}

pub fn mulh(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let a = cpu.regs.read(rs1) as i64 as i128;
    let b = cpu.regs.read(rs2) as i64 as i128;
    let result = a * b;
    cpu.regs.write(rd, ((result >> 64) as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn mulhsu(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let a = cpu.regs.read(rs1) as i64 as i128;
    let b = cpu.regs.read(rs2) as u128;
    let result = a * (b as i128);
    cpu.regs.write(rd, ((result >> 64) as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn divu(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1);
    let divisor = cpu.regs.read(rs2);
    let result = if divisor == 0 { u64::MAX } else { dividend / divisor };
    cpu.regs.write(rd, result);

    Ok(ExecFlow::Next)
}

pub fn div(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as i64;
    let divisor = cpu.regs.read(rs2) as i64;
    let result = if divisor == 0 {
        -1
    } else if dividend == i64::MIN && divisor == -1 {
        i64::MIN
    } else {
        dividend / divisor
    };
    cpu.regs.write(rd, result as u64);

    Ok(ExecFlow::Next)
}

pub fn rem(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as i64;
    let divisor = cpu.regs.read(rs2) as i64;
    let result = if divisor == 0 {
        dividend
    } else if dividend == i64::MIN && divisor == -1 {
        0
    } else {
        dividend % divisor
    };
    cpu.regs.write(rd, result as u64);

    Ok(ExecFlow::Next)
}

pub fn remu(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1);
    let divisor = cpu.regs.read(rs2);
    let result = if divisor == 0 { dividend } else { dividend % divisor };
    cpu.regs.write(rd, result);

    Ok(ExecFlow::Next)
}

pub fn mulw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let a = cpu.regs.read(rs1) as i32;
    let b = cpu.regs.read(rs2) as i32;
    let result = a.wrapping_mul(b);
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

pub fn divw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as i32;
    let divisor = cpu.regs.read(rs2) as i32;
    let result = if divisor == 0 {
        -1
    } else if dividend == i32::MIN && divisor == -1 {
        i32::MIN
    } else {
        dividend / divisor
    };
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

pub fn divuw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as u32;
    let divisor = cpu.regs.read(rs2) as u32;
    let result = if divisor == 0 { u32::MAX } else { dividend / divisor };
    cpu.regs.write(rd, (result as i32 as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn remw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as i32;
    let divisor = cpu.regs.read(rs2) as i32;
    let result = if divisor == 0 {
        dividend
    } else if dividend == i32::MIN && divisor == -1 {
        0
    } else {
        dividend % divisor
    };
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

pub fn remuw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let dividend = cpu.regs.read(rs1) as u32;
    let divisor = cpu.regs.read(rs2) as u32;
    let result = if divisor == 0 { dividend } else { dividend % divisor };
    cpu.regs.write(rd, (result as i32 as i64) as u64);

    Ok(ExecFlow::Next)
}
