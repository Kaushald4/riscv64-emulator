use glasshart_emulator::{
    cpu::{
        Cpu, PrivilegeMode,
        csr::{CSR_MSTATUS, CSR_SSTATUS},
        execute::{csr_execute, system},
        register::Reg,
    },
    trap::Trap,
};

#[test]
fn user_cannot_access_machine_csr() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::User;

    let result = csr_execute::csrrw(&mut cpu, Reg::new(1), Reg::new(2), CSR_MSTATUS);

    assert!(matches!(result, Err(Trap::IllegalInstruction(_))));
}

#[test]
fn supervisor_cannot_access_machine_csr() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let result = csr_execute::csrrw(&mut cpu, Reg::new(1), Reg::new(2), CSR_MSTATUS);

    assert!(matches!(result, Err(Trap::IllegalInstruction(_))));
}

#[test]
fn supervisor_can_access_supervisor_csr() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let result = csr_execute::csrrw(&mut cpu, Reg::new(1), Reg::new(2), CSR_SSTATUS);

    assert!(result.is_ok());
}

#[test]
fn machine_can_access_everything() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Machine;

    assert!(csr_execute::csrrw(&mut cpu, Reg::new(1), Reg::new(2), CSR_MSTATUS).is_ok());

    assert!(csr_execute::csrrw(&mut cpu, Reg::new(1), Reg::new(2), CSR_SSTATUS).is_ok());
}
