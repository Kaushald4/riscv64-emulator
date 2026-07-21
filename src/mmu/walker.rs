use crate::{
    cpu::{Cpu, PrivilegeMode},
    mmu::{
        access_type::AccessType,
        address::VirtualAddress,
        pte::Pte,
        satp::{Satp, SatpMode},
    },
    trap::Trap,
};

pub struct PageWalker;

impl PageWalker {
    const PAGE_SHIFT: u64 = 12;
    const PTE_SIZE: u64 = 8;

    /// Translate a virtual address to physical. Returns the physical address.
    ///
    /// This is the hottest function in the emulator — called 2-3x per
    /// instruction (fetch + load/store). The TLB hit path must be as
    /// lean as possible:
    ///   1. Check if paging is enabled (raw satp, no struct)
    ///   2. Hash VPN, index into TLB with a REFERENCE (no copy)
    ///   3. Check valid + ASID + VPN match
    ///   4. Fast permission check (inline, not a function call)
    ///   5. Compute PA, return
    ///
    /// The slow path (page table walk) is unchanged — it's rare.
    pub fn translate(cpu: &mut Cpu, virtual_address: u64, access: AccessType) -> Result<u64, Trap> {
        // ── Fast bypass: M-mode with no MPRV (or MPRV=0) ──────────────
        // During boot, the guest runs in M-mode with paging off. This
        // check is just a privilege + MPRV bit test — very cheap.
        if cpu.privilege_mode == PrivilegeMode::Machine {
            if access == AccessType::Instruction || !cpu.csr.mprv() {
                return Ok(virtual_address);
            }
        }

        // ── Check if paging is enabled (raw satp, no struct) ──────────
        // satp bits: [63:60] = mode, [59:44] = ASID, [43:0] = PPN
        // Mode 0 = Bare (no translation), Mode 8 = Sv39
        let satp_raw = cpu.csr.satp;
        let satp_mode = (satp_raw >> 60) & 0xF;
        if satp_mode == 0 {
            // Bare — no translation
            return Ok(virtual_address);
        }
        // Sv39 only (mode 8). Other modes are reserved.
        // Don't check is_canonical here — it's rare and the page walk
        // will catch it via invalid PTEs.

        // ── TLB lookup (fast path) ───────────────────────────────────
        let vpn = virtual_address >> 12;
        let asid = ((satp_raw >> 44) & 0xFFFF) as u16;
        let index = (vpn as usize) & 0xFFF;

        // Use a REFERENCE — the old code did `let entry = cpu.tlb.entries[index]`
        // which copies the 56-byte TlbEntry struct on EVERY call. At 90M
        // translate calls/sec, that's 5 GB/s of wasted copies.
        let entry = &cpu.tlb.entries[index];

        if entry.valid && (entry.is_global || entry.asid == asid) {
            let mask_vpn = vpn & !(entry.page_mask >> 12);
            let entry_vpn = entry.vpn & !(entry.page_mask >> 12);

            if mask_vpn == entry_vpn {
                // ── TLB HIT — inline permission check ──────────────────
                //
                // The old code called check_permissions() on every hit,
                // which itself called effective_privilege() — another
                // function call with MPRV checking. Together these were
                // 3.3s + 0.5s = 3.8s of the 70s runtime (5.4%).
                //
                // Inline the common cases:
                //   - S-mode: check SUM if accessing U pages
                //   - U-mode: must be U page
                //   - Instruction: must have X bit
                //   - Read: must have R bit (or X if MXR)
                //   - Write: must have W bit
                //
                // For the TLB hit path, we skip the `accessed` bit check
                // — it was already set when the entry was cached, and
                // the kernel doesn't clear it without flushing the TLB.

                let pte_bits = entry.pte_bits;
                let is_user_page = (pte_bits & (1 << 4)) != 0; // U bit

                // Privilege check (inlined effective_privilege)
                let eff_priv = if cpu.privilege_mode == PrivilegeMode::Machine && access != AccessType::Instruction && cpu.csr.mprv() {
                    cpu.csr.mpp()
                } else {
                    cpu.privilege_mode
                };

                match eff_priv {
                    PrivilegeMode::Machine => {
                        // M-mode can access anything — no permission check
                    }
                    PrivilegeMode::Supervisor => {
                        if is_user_page && access != AccessType::Instruction && !cpu.csr.sum() {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                        if is_user_page && access == AccessType::Instruction {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                    }
                    PrivilegeMode::User => {
                        if !is_user_page {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                    }
                }

                // Access type check (inlined)
                match access {
                    AccessType::Instruction => {
                        if (pte_bits & (1 << 3)) == 0 { // X bit
                            return Err(Self::page_fault(access, virtual_address));
                        }
                    }
                    AccessType::Read => {
                        let readable = if cpu.csr.mxr() {
                            (pte_bits & (1 << 1)) != 0 || (pte_bits & (1 << 3)) != 0 // R or X
                        } else {
                            (pte_bits & (1 << 1)) != 0 // R bit
                        };
                        if !readable {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                    }
                    AccessType::Write => {
                        if (pte_bits & (1 << 2)) == 0 { // W bit
                            return Err(Self::page_fault(access, virtual_address));
                        }
                    }
                }

                // Compute physical address
                let pa = (entry.ppn << 12) | (virtual_address & entry.page_mask);
                return Ok(pa);
            }
        }

        // ── Slow path: page table walk ──────────────────────────────
        let satp = Satp::new(satp_raw);
        let va = VirtualAddress::new(virtual_address);

        if !va.is_canonical() {
            return Err(Self::page_fault(access, virtual_address));
        }

        let mut level: i32 = 2;
        let mut table = satp.ppn() << Self::PAGE_SHIFT;

        loop {
            let walk_vpn = match level {
                2 => va.vpn2() as u64,
                1 => va.vpn1() as u64,
                0 => va.vpn0() as u64,
                _ => unreachable!(),
            };

            let pte_addr = table + walk_vpn * Self::PTE_SIZE;
            let pte = Pte::new(cpu.bus.read64(pte_addr)?);

            if pte.is_invalid() {
                return Err(Self::page_fault(access, virtual_address));
            }

            if pte.is_leaf() {
                Self::check_permissions(cpu, pte, access, virtual_address)?;

                let mut pte_bits = pte.bits();
                let mut pte_updated = false;

                if (pte_bits & (1 << 6)) == 0 {
                    pte_bits |= 1 << 6;
                    pte_updated = true;
                }
                if access == AccessType::Write && (pte_bits & (1 << 7)) == 0 {
                    pte_bits |= 1 << 7;
                    pte_updated = true;
                }

                if pte_updated {
                    cpu.bus.write64(pte_addr, pte_bits)?;
                }

                let page_mask = match level {
                    0 => 0xFFF,
                    1 => {
                        if pte.ppn0() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                        0x1F_FFFF
                    }
                    2 => {
                        if pte.ppn0() != 0 || pte.ppn1() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                        0x3FFF_FFFF
                    }
                    _ => unreachable!(),
                };

                let pa = (pte.ppn() << 12) | (virtual_address & page_mask);

                cpu.tlb.entries[index] = crate::mmu::tlb::TlbEntry {
                    vpn,
                    asid,
                    is_global: (pte_bits & (1 << 5)) != 0,
                    valid: true,
                    pte_bits,
                    page_mask,
                    ppn: pte.ppn(),
                };

                return Ok(pa);
            }

            level -= 1;
            if level < 0 {
                return Err(Self::page_fault(access, virtual_address));
            }

            table = pte.ppn() << Self::PAGE_SHIFT;
        }
    }

    #[inline]
    fn page_fault(access: AccessType, addr: u64) -> Trap {
        match access {
            AccessType::Instruction => Trap::InstructionPageFault(addr),
            AccessType::Read => Trap::LoadPageFault(addr),
            AccessType::Write => Trap::StorePageFault(addr),
        }
    }

    #[inline]
    fn effective_privilege(cpu: &Cpu, access: AccessType) -> PrivilegeMode {
        if cpu.privilege_mode == PrivilegeMode::Machine && access != AccessType::Instruction && cpu.csr.mprv() { cpu.csr.mpp() } else { cpu.privilege_mode }
    }

    #[inline]
    fn check_permissions(cpu: &Cpu, pte: Pte, access: AccessType, virtual_address: u64) -> Result<(), Trap> {
        use crate::cpu::PrivilegeMode;

        match Self::effective_privilege(cpu, access) {
            PrivilegeMode::Machine => {}

            PrivilegeMode::Supervisor => {
                if pte.user() {
                    match access {
                        AccessType::Instruction => {
                            return Err(Self::page_fault(access, virtual_address));
                        }

                        AccessType::Read | AccessType::Write => {
                            if !cpu.csr.sum() {
                                return Err(Self::page_fault(access, virtual_address));
                            }
                        }
                    }
                }
            }

            PrivilegeMode::User => {
                if !pte.user() {
                    return Err(Self::page_fault(access, virtual_address));
                }
            }
        }

        match access {
            AccessType::Instruction => {
                if !pte.execute() || !pte.accessed() {
                    return Err(Self::page_fault(access, virtual_address));
                }
            }

            AccessType::Read => {
                let readable = if cpu.csr.mxr() { pte.read() || pte.execute() } else { pte.read() };

                if !readable || !pte.accessed() {
                    return Err(Self::page_fault(access, virtual_address));
                }
            }

            AccessType::Write => {
                if !pte.write() || !pte.accessed() || !pte.dirty() {
                    return Err(Self::page_fault(access, virtual_address));
                }
            }
        }

        Ok(())
    }
}
