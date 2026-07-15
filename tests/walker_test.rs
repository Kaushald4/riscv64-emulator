use glasshart_emulator::{
    cpu::{Cpu, PrivilegeMode},
    mmu::{access_type::AccessType, walker::PageWalker},
    trap::Trap,
};

#[test]
fn mprv_user_permissions() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Machine;

    cpu.csr.mstatus |= 1 << 17; // MPRV
    cpu.csr.mstatus &= !(0b11 << 11); // MPP = User

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // Supervisor page
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xC3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn machine_mode_without_mprv() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Machine;
    cpu.csr.mstatus &= !(1 << 17);

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xC3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, page);
}

#[test]
fn mxr_enabled_execute_only_page() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;
    cpu.csr.sstatus |= 1 << 19;

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | X | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xC9).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, page);
}

#[test]
fn mxr_disabled_execute_only_page() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;
    cpu.csr.sstatus &= !(1 << 19);

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | X | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xC9).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn supervisor_sum_disabled() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;
    cpu.csr.sstatus &= !(1 << 18);

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | U | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xD3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn supervisor_sum_enabled() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;
    cpu.csr.sstatus |= 1 << 18;

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | U | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xD3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, page);
}

#[test]
fn invalid_pte_load_page_fault() {
    let mut cpu = Cpu::default();

    let root = 0x8000_0000u64;

    // satp.ppn = root >> 12
    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    // Invalid PTE (all zeros)
    cpu.bus.write64(root, 0).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn read_permission_denied() {
    let mut cpu = Cpu::default();
    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000u64;
    let l1 = 0x8000_1000u64;
    let l0 = 0x8000_2000u64;
    let page = 0x8000_3000u64;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | X (no read permission)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0b1001).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn execute_permission_denied() {
    let mut cpu = Cpu::default();
    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000u64;
    let l1 = 0x8000_1000u64;
    let l0 = 0x8000_2000u64;
    let page = 0x8000_3000u64;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R (no execute permission)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0b0011).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Instruction);

    assert!(matches!(result, Err(Trap::InstructionPageFault(0))));
}

#[test]
fn write_permission_denied() {
    let mut cpu = Cpu::default();
    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000u64;
    let l1 = 0x8000_1000u64;
    let l0 = 0x8000_2000u64;
    let page = 0x8000_3000u64;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R (no write permission)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0b0011).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Write);

    assert!(matches!(result, Err(Trap::StorePageFault(0))));
}

#[test]
fn accessed_bit_missing() {
    let mut cpu = Cpu::default();
    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000u64;
    let l1 = 0x8000_1000u64;
    let l0 = 0x8000_2000u64;
    let page = 0x8000_3000u64;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | D (A missing)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0x83).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn dirty_bit_missing() {
    let mut cpu = Cpu::default();
    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000u64;
    let l1 = 0x8000_1000u64;
    let l0 = 0x8000_2000u64;
    let page = 0x8000_3000u64;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | A (D missing)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0x43).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Write);

    assert!(matches!(result, Err(Trap::StorePageFault(0))));
}

#[test]
fn translate_1gb_page() {
    let mut cpu = Cpu::default();

    let root = 0x8000_0000;
    let page = 0xC000_0000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((page >> 12) << 10) | 0xC3).unwrap();

    let va = 0x1234567;

    let result = PageWalker::translate(&mut cpu, va, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, page | (va & 0x3fff_ffff));
}

#[test]
fn misaligned_2mb_page_fault() {
    let mut cpu = Cpu::default();

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let page = 0x8020_1000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();

    cpu.bus.write64(l1, ((page >> 12) << 10) | 0xC3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn non_canonical_load_fault() {
    let mut cpu = Cpu::default();

    cpu.csr.satp = (8u64 << 60) | (0x8000_0000u64 >> 12);

    let va = 0x0000_0040_0000_0000u64;

    let result = PageWalker::translate(&mut cpu, va, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(_))));
}

#[test]
fn non_canonical_instruction_fault() {
    let mut cpu = Cpu::default();

    cpu.csr.satp = (8u64 << 60) | (0x8000_0000u64 >> 12);

    let va = 0x0000_0040_0000_0000u64;

    let result = PageWalker::translate(&mut cpu, va, AccessType::Instruction);

    assert!(matches!(result, Err(Trap::InstructionPageFault(_))));
}

#[test]
fn non_canonical_store_fault() {
    let mut cpu = Cpu::default();

    cpu.csr.satp = (8u64 << 60) | (0x8000_0000u64 >> 12);

    let va = 0x0000_0040_0000_0000u64;

    let result = PageWalker::translate(&mut cpu, va, AccessType::Write);

    assert!(matches!(result, Err(Trap::StorePageFault(_))));
}

#[test]
fn supervisor_cannot_access_user_page() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::Supervisor;

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | U | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xD3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn user_cannot_access_supervisor_page() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::User;

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | A | D (U=0)
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xC3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read);

    assert!(matches!(result, Err(Trap::LoadPageFault(0))));
}

#[test]
fn user_can_access_user_page() {
    let mut cpu = Cpu::default();

    cpu.privilege_mode = PrivilegeMode::User;

    let root = 0x8000_0000;
    let l1 = 0x8000_1000;
    let l0 = 0x8000_2000;
    let page = 0x8000_3000;

    cpu.csr.satp = (8u64 << 60) | (root >> 12);

    cpu.bus.write64(root, ((l1 >> 12) << 10) | 1).unwrap();
    cpu.bus.write64(l1, ((l0 >> 12) << 10) | 1).unwrap();

    // V | R | U | A | D
    cpu.bus.write64(l0, ((page >> 12) << 10) | 0xD3).unwrap();

    let result = PageWalker::translate(&mut cpu, 0, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, page);
}

#[test]
fn bare_mode_identity_mapping() {
    let mut cpu = Cpu::default();

    cpu.csr.satp = 0;

    let va = 0x1234_5678_9abc_def0;

    let result = PageWalker::translate(&mut cpu, va, AccessType::Read).unwrap();

    assert_eq!(result.physical_address, va);
    assert!(!result.translated);
}

#[test]
#[should_panic]
fn reserved_satp_mode_panics() {
    let mut cpu = Cpu::default();

    cpu.csr.satp = 15u64 << 60;

    let _ = PageWalker::translate(&mut cpu, 0, AccessType::Read);
}
