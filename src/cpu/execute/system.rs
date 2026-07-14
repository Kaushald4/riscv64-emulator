use crate::{
    cpu::{
        Cpu, ExecFlow, ExecResult, PrivilegeMode,
        csr::{MSTATUS_MIE, MSTATUS_MPIE, MSTATUS_MPP_MASK, MSTATUS_MPP_SHIFT, MSTATUS_SIE, MSTATUS_SPIE, MSTATUS_SPP},
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
    // read previous privilege mode from MPP.
    let mpp = (cpu.csr.mstatus & MSTATUS_MPP_MASK) >> MSTATUS_MPP_SHIFT;

    cpu.privilege_mode = match mpp {
        0 => PrivilegeMode::User,
        1 => PrivilegeMode::Supervisor,
        3 => PrivilegeMode::Machine,
        _ => return Err(Trap::IllegalInstruction(0)),
    };

    // MIE <- MPIE
    let mpie = (cpu.csr.mstatus & MSTATUS_MPIE) != 0;

    if mpie {
        cpu.csr.mstatus |= MSTATUS_MIE;
    } else {
        cpu.csr.mstatus &= !MSTATUS_MIE;
    }

    // MPIE <- 1
    cpu.csr.mstatus |= MSTATUS_MPIE;

    // MPP <- User (00)
    cpu.csr.mstatus &= !MSTATUS_MPP_MASK;

    // return to the saved PC.
    Ok(ExecFlow::Jump(cpu.csr.mepc))
}

pub fn sret(cpu: &mut Cpu) -> ExecResult {
    // SPP == 0 -> User
    // SPP == 1 -> Supervisor
    let spp = (cpu.csr.mstatus & MSTATUS_SPP) != 0;

    cpu.privilege_mode = if spp { PrivilegeMode::Supervisor } else { PrivilegeMode::User };

    // SIE <- SPIE
    let spie = (cpu.csr.mstatus & MSTATUS_SPIE) != 0;

    if spie {
        cpu.csr.mstatus |= MSTATUS_SIE;
    } else {
        cpu.csr.mstatus &= !MSTATUS_SIE;
    }

    // SPIE <- 1
    cpu.csr.mstatus |= MSTATUS_SPIE;

    // SPP <- 0
    cpu.csr.mstatus &= !MSTATUS_SPP;

    Ok(ExecFlow::Jump(cpu.csr.sepc))
}
