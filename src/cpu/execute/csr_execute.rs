use crate::cpu::{Cpu, ExecFlow, execute::ExecResult, register::Reg};

pub fn csrrw(cpu: &mut Cpu, rd: Reg, rs1: Reg, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.csr.write(csr, cpu.regs.read(rs1))?;

    cpu.regs.write(rd, old);

    Ok(ExecFlow::Next)
}

pub fn csrrs(cpu: &mut Cpu, rd: Reg, rs1: Reg, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.regs.write(rd, old);

    if !rs1.is_zero() {
        cpu.csr.write(csr, old | cpu.regs.read(rs1))?;
    }

    Ok(ExecFlow::Next)
}

pub fn csrrc(cpu: &mut Cpu, rd: Reg, rs1: Reg, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.regs.write(rd, old);

    if !rs1.is_zero() {
        cpu.csr.write(csr, old & !cpu.regs.read(rs1))?;
    }

    Ok(ExecFlow::Next)
}

pub fn csrrwi(cpu: &mut Cpu, rd: Reg, uimm: u8, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.csr.write(csr, uimm as u64)?;

    cpu.regs.write(rd, old);

    Ok(ExecFlow::Next)
}

pub fn csrrsi(cpu: &mut Cpu, rd: Reg, uimm: u8, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.regs.write(rd, old);

    if uimm != 0 {
        cpu.csr.write(csr, old | uimm as u64)?;
    }

    Ok(ExecFlow::Next)
}

pub fn csrrci(cpu: &mut Cpu, rd: Reg, uimm: u8, csr: u16) -> ExecResult {
    let old = cpu.csr.read(csr)?;

    cpu.regs.write(rd, old);

    if uimm != 0 {
        cpu.csr.write(csr, old & !(uimm as u64))?;
    }

    Ok(ExecFlow::Next)
}
