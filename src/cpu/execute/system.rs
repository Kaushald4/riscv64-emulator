use crate::{
    cpu::{Cpu, ExecFlow, ExecResult, PrivilegeMode},
    trap::Trap,
};
pub fn fence(_cpu: &mut Cpu, _pred: u8, _succ: u8, _fm: u8) -> ExecResult {
    Ok(ExecFlow::Next)
}

pub fn fence_i(_cpu: &mut Cpu) -> ExecResult {
    // No instruction cache yet.
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
