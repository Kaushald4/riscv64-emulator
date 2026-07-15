use glasshart_emulator::cpu::csr::Csr;

#[test]
fn medeleg_helper() {
    let mut csr = Csr::default();

    csr.medeleg = 1 << 13;

    assert!(csr.is_exception_delegated(13));
    assert!(!csr.is_exception_delegated(12));
}

#[test]
fn mideleg_helper() {
    let mut csr = Csr::default();

    csr.mideleg = 1 << 5;

    assert!(csr.is_interrupt_delegated(5));
    assert!(!csr.is_interrupt_delegated(7));
}
