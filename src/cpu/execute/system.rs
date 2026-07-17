use crate::{
    cpu::{
        Cpu, ExecFlow, ExecResult, PrivilegeMode,
        csr::{MSTATUS_MIE, MSTATUS_MPIE, MSTATUS_MPP_MASK, MSTATUS_MPP_SHIFT},
        register::Reg,
    },
    trap::Trap,
};

pub fn fence(_cpu: &mut Cpu, _pred: u8, _succ: u8, _fm: u8) -> ExecResult {
    Ok(ExecFlow::Next)
}

pub fn fence_i(_cpu: &mut Cpu) -> ExecResult {
    // no instruction cache yet.
    Ok(ExecFlow::Next)
}

pub fn ebreak(_cpu: &mut Cpu) -> ExecResult {
    Err(Trap::Breakpoint)
}

pub fn ecall(cpu: &mut Cpu) -> ExecResult {
    match cpu.privilege_mode {
        PrivilegeMode::User => Err(Trap::EcallFromUMode),
        PrivilegeMode::Supervisor => Err(Trap::EcallFromSMode),
        PrivilegeMode::Machine => Err(Trap::EcallFromMMode),
    }
}

pub fn mret(cpu: &mut Cpu) -> ExecResult {
    let mpp = (cpu.csr.mstatus & MSTATUS_MPP_MASK) >> MSTATUS_MPP_SHIFT;

    cpu.privilege_mode = match mpp {
        0 => PrivilegeMode::User,
        1 => PrivilegeMode::Supervisor,
        3 => PrivilegeMode::Machine,
        _ => return Err(Trap::IllegalInstruction(0)),
    };

    // MIE <- MPIE
    if (cpu.csr.mstatus & MSTATUS_MPIE) != 0 {
        cpu.csr.mstatus |= MSTATUS_MIE;
    } else {
        cpu.csr.mstatus &= !MSTATUS_MIE;
    }

    // MPIE <- 1
    cpu.csr.mstatus |= MSTATUS_MPIE;

    // MPP <- User
    cpu.csr.mstatus &= !MSTATUS_MPP_MASK;

    Ok(ExecFlow::Jump(cpu.csr.mepc))
}

pub fn sret(cpu: &mut Cpu) -> ExecResult {
    // SPP indicates the privilege mode to return to.
    let spp = (cpu.csr.mstatus >> 8) & 1;

    cpu.privilege_mode = if spp == 0 { PrivilegeMode::User } else { PrivilegeMode::Supervisor };

    // SIE <- SPIE
    let spie = (cpu.csr.mstatus >> 5) & 1;

    cpu.csr.mstatus &= !(1 << 1);
    cpu.csr.mstatus |= spie << 1;

    // SPIE <- 1
    cpu.csr.mstatus |= 1 << 5;

    // SPP <- User
    cpu.csr.mstatus &= !(1 << 8);

    Ok(ExecFlow::Jump(cpu.csr.sepc))
}

pub fn sfence_vma(cpu: &mut Cpu, rs1: Reg, rs2: Reg) -> ExecResult {
    let vaddr = cpu.regs.read(rs1);
    let asid = cpu.regs.read(rs2) as u16;

    if vaddr == 0 && asid == 0 {
        // SFENCE.VMA x0, x0
        cpu.tlb.flush_all();
    } else if vaddr == 0 && asid != 0 {
        // SFENCE.VMA x0, rs2
        cpu.tlb.flush_asid(asid);
    } else if vaddr != 0 && asid == 0 {
        // SFENCE.VMA rs1, x0
        cpu.tlb.flush_page(vaddr, 0, true);
    } else {
        // SFENCE.VMA rs1, rs2
        cpu.tlb.flush_page(vaddr, asid, false);
    }

    Ok(ExecFlow::Next)
}

pub fn wfi(_cpu: &mut Cpu) -> ExecResult {
    // no until interrupts are implemented.
    Ok(ExecFlow::Next)
}

pub fn required_privilege(csr: u16) -> PrivilegeMode {
    match (csr >> 8) & 0b11 {
        0 => PrivilegeMode::User,
        1 => PrivilegeMode::Supervisor,
        2 => unreachable!(),
        3 => PrivilegeMode::Machine,
        _ => unreachable!(),
    }
}

pub fn has_csr_access(current: PrivilegeMode, csr: u16) -> bool {
    match current {
        PrivilegeMode::Machine => true,

        PrivilegeMode::Supervisor => {
            matches!(required_privilege(csr), PrivilegeMode::Supervisor | PrivilegeMode::User)
        }

        PrivilegeMode::User => {
            matches!(required_privilege(csr), PrivilegeMode::User)
        }
    }
}
