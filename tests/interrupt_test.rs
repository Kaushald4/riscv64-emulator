use glasshart_emulator::cpu::{
    Cpu, PrivilegeMode,
    csr::{CSR_MIE, CSR_MIP, CSR_SIE, CSR_SIP, Csr, SIE_MASK, SIP_MASK},
};

#[test]
fn pending_machine_timer_interrupt() {
    let mut cpu = Cpu::default();

    cpu.csr.mie |= 1 << 7;
    cpu.csr.mip |= 1 << 7;

    assert_eq!(cpu.pending_interrupt(), Some(7));
}

#[test]
fn machine_timer_interrupt_goes_to_mtvec() {
    let mut cpu = Cpu::default();

    cpu.pc = 0x8000_1000;
    cpu.csr.mtvec = 0x8000_2000;

    cpu.handle_interrupt(7).unwrap();

    assert_eq!(cpu.pc, 0x8000_2000);
    assert_eq!(cpu.csr.mepc, 0x8000_1000);
    assert_eq!(cpu.csr.mcause, (1u64 << 63) | 7);
}

#[test]
fn delegated_interrupt_goes_to_stvec() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    cpu.pc = 0x8000_1000;

    cpu.csr.stvec = 0x8000_3000;
    cpu.csr.mideleg = 1 << 7;

    cpu.handle_interrupt(7).unwrap();

    assert_eq!(cpu.pc, 0x8000_3000);
    assert_eq!(cpu.csr.sepc, 0x8000_1000);
    assert_eq!(cpu.csr.scause, (1u64 << 63) | 7);
}

#[test]
fn sie_aliases_mie() {
    let mut csr = Csr::default();

    csr.write(CSR_SIE, 0x222).unwrap();

    assert_eq!(csr.read(CSR_SIE).unwrap(), 0x222 & SIE_MASK);
    assert_eq!(csr.read(CSR_MIE).unwrap() & SIE_MASK, 0x222 & SIE_MASK);
}

#[test]
fn sip_aliases_mip() {
    let mut csr = Csr::default();

    csr.write(CSR_SIP, 0x111).unwrap();

    assert_eq!(csr.read(CSR_SIP).unwrap(), 0x111 & SIP_MASK);
    assert_eq!(csr.read(CSR_MIP).unwrap() & SIP_MASK, 0x111 & SIP_MASK);
}
