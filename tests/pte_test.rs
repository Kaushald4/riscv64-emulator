use glasshart_emulator::mmu::pte::Pte;

#[test]
fn empty_pte() {
    let pte = Pte::new(0);

    assert!(!pte.valid());
    assert!(!pte.read());
    assert!(!pte.write());
    assert!(!pte.execute());
    assert!(!pte.user());
    assert!(!pte.global());
    assert!(!pte.accessed());
    assert!(!pte.dirty());

    assert!(pte.is_invalid());
    assert!(!pte.is_leaf());
}

#[test]
fn pointer_pte() {
    let bits = 1u64 << 0;

    let pte = Pte::new(bits);

    assert!(pte.valid());
    assert!(!pte.read());
    assert!(!pte.write());
    assert!(!pte.execute());

    assert!(!pte.is_invalid());
    assert!(!pte.is_leaf());
}

#[test]
fn leaf_read() {
    let bits = (1u64 << 0) | (1u64 << 1);

    let pte = Pte::new(bits);

    assert!(pte.valid());
    assert!(pte.read());
    assert!(pte.is_leaf());
    assert!(!pte.is_invalid());
}

#[test]
fn leaf_execute() {
    let bits = (1u64 << 0) | (1u64 << 3);

    let pte = Pte::new(bits);

    assert!(pte.valid());
    assert!(pte.execute());
    assert!(pte.is_leaf());
    assert!(!pte.is_invalid());
}

#[test]
fn invalid_rw_combination() {
    let bits = (1u64 << 0) | (1u64 << 2);

    let pte = Pte::new(bits);

    assert!(pte.valid());
    assert!(!pte.read());
    assert!(pte.write());

    assert!(pte.is_invalid());
}

#[test]
fn all_flags() {
    let bits = (1u64 << 0) | (1u64 << 1) | (1u64 << 2) | (1u64 << 3) | (1u64 << 4) | (1u64 << 5) | (1u64 << 6) | (1u64 << 7);

    let pte = Pte::new(bits);

    assert!(pte.valid());
    assert!(pte.read());
    assert!(pte.write());
    assert!(pte.execute());
    assert!(pte.user());
    assert!(pte.global());
    assert!(pte.accessed());
    assert!(pte.dirty());

    assert!(pte.is_leaf());
    assert!(!pte.is_invalid());
}

#[test]
fn ppn_extraction() {
    let ppn0 = 0x155u64;
    let ppn1 = 0x0aau64;
    let ppn2 = 0x123456u64;

    let bits = (ppn0 << 10) | (ppn1 << 19) | (ppn2 << 28) | 1;

    let pte = Pte::new(bits);

    assert_eq!(pte.ppn0(), ppn0 as u16);
    assert_eq!(pte.ppn1(), ppn1 as u16);
    assert_eq!(pte.ppn2(), ppn2 as u32);

    let expected = ppn0 | (ppn1 << 9) | (ppn2 << 18);

    assert_eq!(pte.ppn(), expected);
}
