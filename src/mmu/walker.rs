use crate::{
    cpu::{Cpu, PrivilegeMode},
    mmu::{
        access_type::AccessType,
        address::VirtualAddress,
        pte::Pte,
        satp::{Satp, SatpMode},
        translation::Translation,
    },
    trap::Trap,
};

pub struct PageWalker;

impl PageWalker {
    const PAGE_SHIFT: u64 = 12;
    const PTE_SIZE: u64 = 8;

    pub fn translate(cpu: &mut Cpu, virtual_address: u64, access: AccessType) -> Result<Translation, Trap> {
        let satp = Satp::new(cpu.csr.satp);

        // 1. Bypass MMU completely for Machine Mode (or M-mode with MPRV=0)
        if Self::effective_privilege(cpu, access) == PrivilegeMode::Machine {
            return Ok(Translation {
                physical_address: virtual_address,
                translated: false,
                root_page_table: 0,
            });
        }

        // 2. Otherwise, check SATP for Supervisor/User translation
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

        let mut level: i32 = 2;
        let mut table = satp.ppn() << Self::PAGE_SHIFT;

        loop {
            let vpn = match level {
                2 => va.vpn2() as u64,
                1 => va.vpn1() as u64,
                0 => va.vpn0() as u64,
                _ => unreachable!(),
            };

            let pte_addr = table + vpn * Self::PTE_SIZE;
            let pte = Pte::new(cpu.bus.read64(pte_addr)?);

            if pte.is_invalid() {
                return Err(Self::page_fault(access, virtual_address));
            }

            if pte.is_leaf() {
                Self::check_permissions(cpu, pte, access, virtual_address)?;

                let pa = match level {
                    // 4 KiB page
                    0 => (pte.ppn() << 12) | (virtual_address & 0xfff),

                    // 2 MiB page
                    1 => {
                        // misaligned superpage?
                        if pte.ppn0() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }

                        (pte.ppn() << 12) | (virtual_address & 0x1f_ffff)
                    }

                    // 1 GiB page
                    2 => {
                        // misaligned superpage?
                        if pte.ppn0() != 0 || pte.ppn1() != 0 {
                            return Err(Self::page_fault(access, virtual_address));
                        }

                        (pte.ppn() << 12) | (virtual_address & 0x3fff_ffff)
                    }

                    _ => unreachable!(),
                };

                return Ok(Translation {
                    physical_address: pa,
                    translated: true,
                    root_page_table: satp.ppn() << Self::PAGE_SHIFT,
                });
            }

            // spec step 5:
            // descend to the next level.
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
