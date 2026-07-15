use glasshart_emulator::mmu::address::VirtualAddress;

#[test]
fn decode_zero() {
    let addr = (0x155u64 << 30) | (0x0aau64 << 21) | (0x123u64 << 12) | 0x456;
    let va = VirtualAddress::new(addr);

    assert_eq!(va.offset(), 0x456);
    assert_eq!(va.vpn0(), 0x123);
    assert_eq!(va.vpn1(), 0x0aa);
    assert_eq!(va.vpn2(), 0x155);
    assert!(VirtualAddress::new(0x0000_0000_1234_5678).is_canonical());
    assert!(VirtualAddress::new(0xffff_ffc0_1234_5678).is_canonical());
    assert!(!VirtualAddress::new(0x0000_0040_0000_0000).is_canonical());
}
