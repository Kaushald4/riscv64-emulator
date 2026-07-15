use crate::cpu::{Cpu, ExecFlow, ExecResult, register::Reg};

pub fn lr_w(cpu: &mut Cpu, rd: Reg, rs1: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let value = cpu.bus.read32(addr)? as i32 as i64 as u64;

    cpu.reserve_address(addr);
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn lr_d(cpu: &mut Cpu, rd: Reg, rs1: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let value = cpu.bus.read64(addr)?;

    cpu.reserve_address(addr);
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sc_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    if cpu.reservation_matches(addr) {
        cpu.bus.write32(addr, cpu.regs.read(rs2) as u32)?;
        cpu.regs.write(rd, 0);
    } else {
        cpu.regs.write(rd, 1);
    }
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}

pub fn sc_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    if cpu.reservation_matches(addr) {
        cpu.bus.write64(addr, cpu.regs.read(rs2))?;
        cpu.regs.write(rd, 0);
    } else {
        cpu.regs.write(rd, 1);
    }
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}

// swap
pub fn amoswap_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read32(addr)? as i32 as i64 as u64;
    cpu.bus.write32(addr, cpu.regs.read(rs2) as u32)?;
    cpu.regs.write(rd, old);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}

pub fn amoswap_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read64(addr)?;
    cpu.bus.write64(addr, cpu.regs.read(rs2))?;
    cpu.regs.write(rd, old);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}

// arithmetic
pub fn amoadd_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read32(addr)? as i32;
    let rhs = cpu.regs.read(rs2) as i32;
    let new = old.wrapping_add(rhs);
    cpu.bus.write32(addr, new as u32)?;
    cpu.regs.write(rd, old as i64 as u64);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoadd_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read64(addr)? as i64;
    let rhs = cpu.regs.read(rs2) as i64;
    let new = old.wrapping_add(rhs);
    cpu.bus.write64(addr, new as u64)?;
    cpu.regs.write(rd, old as u64);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoxor_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read32(addr)? as u32;
    let rhs = cpu.regs.read(rs2) as u32;
    cpu.bus.write32(addr, old ^ rhs)?;
    cpu.regs.write(rd, old as i32 as i64 as u64);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoxor_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read64(addr)?;
    let rhs = cpu.regs.read(rs2);
    cpu.bus.write64(addr, old ^ rhs)?;
    cpu.regs.write(rd, old);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoor_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read32(addr)? as u32;
    let rhs = cpu.regs.read(rs2) as u32;
    cpu.bus.write32(addr, old | rhs)?;
    cpu.regs.write(rd, old as i32 as i64 as u64);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoor_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read64(addr)?;
    let rhs = cpu.regs.read(rs2);
    cpu.bus.write64(addr, old | rhs)?;
    cpu.regs.write(rd, old);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoand_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read32(addr)? as u32;
    let rhs = cpu.regs.read(rs2) as u32;
    cpu.bus.write32(addr, old & rhs)?;
    cpu.regs.write(rd, old as i32 as i64 as u64);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amoand_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);
    let old = cpu.bus.read64(addr)?;
    let rhs = cpu.regs.read(rs2);
    cpu.bus.write64(addr, old & rhs)?;
    cpu.regs.write(rd, old);
    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}

// min/max
pub fn amomin_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read32(addr)? as i32;
    let rhs = cpu.regs.read(rs2) as i32;

    let new = old.min(rhs);

    cpu.bus.write32(addr, new as u32)?;
    cpu.regs.write(rd, old as i64 as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amomin_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read64(addr)? as i64;
    let rhs = cpu.regs.read(rs2) as i64;

    let new = old.min(rhs);

    cpu.bus.write64(addr, new as u64)?;
    cpu.regs.write(rd, old as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amomax_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read32(addr)? as i32;
    let rhs = cpu.regs.read(rs2) as i32;

    let new = old.max(rhs);

    cpu.bus.write32(addr, new as u32)?;
    cpu.regs.write(rd, old as i64 as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amomax_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read64(addr)? as i64;
    let rhs = cpu.regs.read(rs2) as i64;

    let new = old.max(rhs);

    cpu.bus.write64(addr, new as u64)?;
    cpu.regs.write(rd, old as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amominu_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read32(addr)? as u32;
    let rhs = cpu.regs.read(rs2) as u32;

    let new = old.min(rhs);

    cpu.bus.write32(addr, new)?;
    cpu.regs.write(rd, old as i32 as i64 as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amominu_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read64(addr)?;
    let rhs = cpu.regs.read(rs2);

    let new = old.min(rhs);

    cpu.bus.write64(addr, new)?;
    cpu.regs.write(rd, old);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amomaxu_w(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read32(addr)? as u32;
    let rhs = cpu.regs.read(rs2) as u32;

    let new = old.max(rhs);

    cpu.bus.write32(addr, new)?;
    cpu.regs.write(rd, old as i32 as i64 as u64);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
pub fn amomaxu_d(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let addr = cpu.regs.read(rs1);

    let old = cpu.bus.read64(addr)?;
    let rhs = cpu.regs.read(rs2);

    let new = old.max(rhs);

    cpu.bus.write64(addr, new)?;
    cpu.regs.write(rd, old);

    cpu.clear_reservation();

    Ok(ExecFlow::Next)
}
