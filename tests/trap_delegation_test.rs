use glasshart_emulator::cpu::{Cpu, PrivilegeMode};
use glasshart_emulator::trap::Trap;

#[test]
fn page_fault_delegated_to_supervisor() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    cpu.pc = 0x8000_1234;

    cpu.csr.stvec = 0x8020_0000;
    cpu.csr.mtvec = 0x8040_0000;

    // Delegate instruction page faults.
    cpu.csr.medeleg = 1 << 12;

    cpu.handle_trap(Trap::InstructionPageFault(0x4000)).unwrap();

    assert_eq!(cpu.privilege_mode, PrivilegeMode::Supervisor);

    assert_eq!(cpu.csr.sepc, 0x8000_1234);
    assert_eq!(cpu.csr.scause, 12);
    assert_eq!(cpu.csr.stval, 0x4000);

    assert_eq!(cpu.pc, 0x8020_0000);

    // Machine CSRs must remain untouched.
    assert_eq!(cpu.csr.mepc, 0);
    assert_eq!(cpu.csr.mcause, 0);
}

#[test]
fn page_fault_not_delegated_goes_to_machine() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    cpu.pc = 0x8000_1234;

    cpu.csr.mtvec = 0x8040_0000;

    cpu.handle_trap(Trap::InstructionPageFault(0x4000)).unwrap();

    assert_eq!(cpu.privilege_mode, PrivilegeMode::Machine);

    assert_eq!(cpu.csr.mepc, 0x8000_1234);
    assert_eq!(cpu.csr.mcause, 12);
    assert_eq!(cpu.csr.mtval, 0x4000);

    assert_eq!(cpu.pc, 0x8040_0000);
}
