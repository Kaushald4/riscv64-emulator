use glasshart_emulator::cpu::{Cpu, ExecFlow, PrivilegeMode, execute::system};

#[test]
fn sret_restores_state() {
    let mut cpu = Cpu::default();

    cpu.csr.sepc = 0x1234;

    cpu.csr.sstatus |= 1 << 8; // SPP = S
    cpu.csr.sstatus |= 1 << 5; // SPIE = 1

    cpu.privilege_mode = PrivilegeMode::Machine;

    let flow = system::sret(&mut cpu).unwrap();

    match flow {
        ExecFlow::Jump(pc) => assert_eq!(pc, 0x1234),
        _ => panic!("expected ExecFlow::Jump"),
    }

    assert_eq!(cpu.privilege_mode, PrivilegeMode::Supervisor);

    // SIE <- SPIE
    assert_eq!((cpu.csr.sstatus >> 1) & 1, 1);

    // SPIE <- 1
    assert_eq!((cpu.csr.sstatus >> 5) & 1, 1);

    // SPP <- 0
    assert_eq!((cpu.csr.sstatus >> 8) & 1, 0);
}

#[test]
fn mret_restores_state() {
    let mut cpu = Cpu::default();

    cpu.csr.mepc = 0x5678;

    cpu.csr.mstatus |= 1 << 7; // MPIE
    cpu.csr.mstatus |= 1 << 11; // MPP = S

    let flow = system::mret(&mut cpu).unwrap();

    match flow {
        ExecFlow::Jump(pc) => assert_eq!(pc, 0x5678),
        _ => panic!("expected ExecFlow::Jump"),
    }

    assert_eq!(cpu.privilege_mode, PrivilegeMode::Supervisor);

    // MIE <- MPIE
    assert_eq!((cpu.csr.mstatus >> 3) & 1, 1);

    // MPIE <- 1
    assert_eq!((cpu.csr.mstatus >> 7) & 1, 1);

    // MPP <- 0
    assert_eq!((cpu.csr.mstatus >> 11) & 0b11, 0);
}
