use crate::{
    cpu::{Cpu, PrivilegeMode},
    mmu::{
        access_type::AccessType,
        address::VirtualAddress,
        pte::Pte,
        satp::{Satp, SatpMode},
        tlb::Tlb,
        translation::Translation,
    },
    trap::Trap,
};

pub struct PageWalker;

impl PageWalker {
    const PAGE_SHIFT: u64 = 12;
    const PTE_SIZE: u64 = 8;

    pub fn translate(cpu: &mut Cpu, virtual_address: u64, access: AccessType) -> Result<Translation, Trap> {
        // bypass MMU completely for machine mode (or M-mode with MPRV=0)
        if Self::effective_privilege(cpu, access) == PrivilegeMode::Machine {
            return Ok(Translation {
                physical_address: virtual_address,
                translated: false,
                root_page_table: 0,
            });
        }

        let satp = Satp::new(cpu.csr.satp);

        match satp.mode() {
            SatpMode::Bare => {
                return Ok(Translation {
                    physical_address: virtual_address,
                    translated: false,
                    root_page_table: 0,
                });
            }
            SatpMode::Reserved(_) => unreachable!(),
            SatpMode::Sv39 => {}
        }

        let va = VirtualAddress::new(virtual_address);

        if !va.is_canonical() {
            return Err(Self::page_fault(access, virtual_address));
        }

        let vpn = virtual_address >> 12;
        let asid = satp.asid() as u16;

        // --- 1. FAST PATH: TLB LOOKUP ---
        let index = crate::mmu::tlb::Tlb::hash(vpn);
        let entry = cpu.tlb.entries[index];

        // Hit condition: Entry is valid AND (it is a Global page OR the ASID matches)
        if entry.valid && (entry.is_global || entry.asid == asid) {
            // Apply the superpage mask to verify the VPN actually matches
            let mask_vpn = vpn & !(entry.page_mask >> 12);
            let entry_vpn = entry.vpn & !(entry.page_mask >> 12);

            if mask_vpn == entry_vpn {
                // Reconstruct the PTE from the cached bits and verify permissions
                let cached_pte = Pte::new(entry.pte_bits);
                Self::check_permissions(cpu, cached_pte, access, virtual_address)?;

                // Calculate final physical address using the cached base PPN and the exact page mask
                let pa = (entry.ppn << 12) | (virtual_address & entry.page_mask);

                return Ok(Translation {
                    physical_address: pa,
                    translated: true,
                    root_page_table: satp.ppn() << Self::PAGE_SHIFT,
                });
            }
        }

        // --- 2. SLOW PATH: HARDWARE PAGE TABLE WALK ---
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

            // Spec step 3:
            if pte.is_invalid() {
                return Err(Self::page_fault(access, virtual_address));
            }

            // leaf PTE found.
            if pte.is_leaf() {
                Self::check_permissions(cpu, pte, access, virtual_address)?;

                // --- 3. HARDWARE A/D BIT UPDATE (CRITICAL FIX) ---
                // If the Accessed (bit 6) or Dirty (bit 7) bits are 0, the hardware MUST set them.
                // Otherwise, Linux will get stuck in an infinite page fault loop!
                let mut pte_bits = pte.bits();
                let mut pte_updated = false;

                // Check Accessed bit (bit 6)
                if (pte_bits & (1 << 6)) == 0 {
                    pte_bits |= 1 << 6;
                    pte_updated = true;
                }
                // Check Dirty bit (bit 7)
                if access == AccessType::Write && (pte_bits & (1 << 7)) == 0 {
                    pte_bits |= 1 << 7;
                    pte_updated = true;
                }

                // Write the updated PTE back to physical memory
                if pte_updated {
                    cpu.bus.write64(pte_addr, pte_bits)?;
                }

                // --- 4. SUPERPAGE MASKING & PA CALCULATION ---
                let page_mask = match level {
                    0 => 0xFFF, // 4 KiB page
                    1 => {
                        // 2 MiB page
                        if pte.ppn0() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                        0x1F_FFFF
                    }
                    2 => {
                        // 1 GiB page
                        if pte.ppn0() != 0 || pte.ppn1() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }
                        0x3FFF_FFFF
                    }
                    _ => unreachable!(),
                };

                // Calculate the exact physical address
                let pa = (pte.ppn() << 12) | (virtual_address & page_mask);

                // --- 5. CACHE FILL: POPULATE THE TLB ---
                cpu.tlb.entries[index] = crate::mmu::tlb::TlbEntry {
                    vpn,
                    asid,
                    is_global: (pte_bits & (1 << 5)) != 0, // Global bit (bit 5)
                    valid: true,
                    pte_bits,
                    page_mask,
                    ppn: pte.ppn(), // Store the raw PPN base, NOT shifted pa
                };

                return Ok(Translation {
                    physical_address: pa,
                    translated: true,
                    root_page_table: satp.ppn() << Self::PAGE_SHIFT,
                });
            }

            // spec step 5: descend to the next level.
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
        use crate::cpu::PrivilegeMode;

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
                // User must access only U pages.
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
