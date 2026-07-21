pub mod bus;
pub mod csr;
pub mod decoder_cache;
pub mod execute;
pub mod f_register;
pub mod memory;
pub mod register;

use crate::{
    cpu::{bus::Bus, csr::Csr, decoder_cache::DecodeEntry},
    decode::decode,
    devices::Device,
    mmu::{Mmu, tlb::Tlb, access_type::AccessType},
    trap::Trap,
};

use f_register::FRegister;
use register::Register;

#[derive(Debug)]
pub enum ExecFlow {
    Next,
    Jump(u64),
}
pub type ExecResult = Result<ExecFlow, Trap>;

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum PrivilegeMode {
    Machine = 0,
    Supervisor = 1,
    User = 2,
}

pub struct Cpu {
    pub regs: Register,
    pub f_regs: FRegister,
    pub pc: u64,
    pub bus: Bus,
    pub current_instruction_length: u8,
    pub privilege_mode: PrivilegeMode,
    pub csr: Csr,
    pub tlb: Tlb,
    pub decode_cache: Box<[DecodeEntry]>,
    // reservation for LR/SC
    pub reservation: Option<u64>,
    pub wfi: bool,

    pub clock: u64,

    // ── Instruction fetch cache ──────────────────────────────────────
    //
    // Caches the physical address of the current code page. For ~1024
    // consecutive instructions within the same 4KB page, we skip the
    // entire MMU translate path.
    //
    // We store the PA of the PC that was translated, plus that PC itself.
    // For subsequent fetches in the same page:
    //   pa = cached_pa + (pc - cached_pc)
    // using wrapping arithmetic (handles backward jumps within a page).
    pub fetch_page_vpn: u64,      // virtual page number (va >> 12)
    pub fetch_page_pa: u64,       // PA of the translated PC
    pub fetch_page_pc: u64,      // the PC that was translated
    pub fetch_page_valid: bool,   // is the cache valid?

    // ── Data access page cache ─────────────────────────────────────
    //
    // Same idea as the fetch page cache but for load/store accesses.
    // Separate caches for read and write because a page might be
    // readable but not writable — we must not skip the permission
    // check for a write just because a read was cached.
    //
    // Each cache stores the VPN, the PA of the cached access, and the
    // VA that was translated. For subsequent accesses in the same
    // page: pa = cached_pa + (addr - cached_va).
    pub data_read_vpn: u64,
    pub data_read_pa: u64,
    pub data_read_va: u64,
    pub data_read_valid: bool,
    pub data_write_vpn: u64,
    pub data_write_pa: u64,
    pub data_write_va: u64,
    pub data_write_valid: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Register::new(),
            f_regs: FRegister::new(),
            pc: 0x8000_0000,
            bus: Bus::new(128 * 1024 * 1024),
            current_instruction_length: 4,
            privilege_mode: PrivilegeMode::Machine,
            csr: Csr::new(),
            tlb: Tlb::new(),
            decode_cache: vec![DecodeEntry::default(); 16384].into_boxed_slice(),
            reservation: None,
            wfi: false,
            clock: 0,
            fetch_page_vpn: 0,
            fetch_page_pa: 0,
            fetch_page_pc: 0,
            fetch_page_valid: false,
            data_read_vpn: 0,
            data_read_pa: 0,
            data_read_va: 0,
            data_read_valid: false,
            data_write_vpn: 0,
            data_write_pa: 0,
            data_write_va: 0,
            data_write_valid: false,
        }
    }

    fn fetch(&mut self) -> Result<u32, Trap> {
        if self.pc & 1 != 0 {
            return Err(Trap::InstructionAddressMisaligned(self.pc));
        }

        // Instruction fetch cache
        //
        // 99% of instructions are in the same 4KB page as the previous
        // instruction. By caching the physical address of the page, we
        // skip the entire MMU translate path (TLB lookup + permission
        // check) for every instruction within a page.
        //
        // This is the #1 performance optimization: translate() was 23%
        // of runtime, and 2/3 of translate calls are from fetch. This
        // cache eliminates those calls entirely for same-page fetches.
        //
        // We must invalidate the cache on:
        //   - Page boundary crossing (pc >> 12 changes)
        //   - TLB flush (sfence.vma)
        //   - Privilege mode change (trap, sret, mret)
        //   - satp write (context switch)
        //
        // For simplicity, we just re-translate on any VPN mismatch.
        // The TLB itself handles the actual caching; this fetch cache
        // just avoids the function call overhead.

        let vpn = self.pc >> 12;
        let offset = (self.pc & 0xFFF) as usize;

        if !self.fetch_page_valid || self.fetch_page_vpn != vpn {
            // Cache miss — translate and cache
            self.fetch_page_pa = Mmu::translate(self, self.pc, AccessType::Instruction)?;
            self.fetch_page_pc = self.pc;
            self.fetch_page_vpn = vpn;
            self.fetch_page_valid = true;
        }

        // Compute PA: cached_pa + (pc - cached_pc) using wrapping arithmetic.
        // This handles both forward and backward jumps within the same page.
        let pa = self.fetch_page_pa.wrapping_add(self.pc.wrapping_sub(self.fetch_page_pc));

        // Read directly from the cached physical address
        if offset <= 0xFFC {
            // 4 bytes fit within the page
            let word = self.bus.read32_fast(pa)?;
            if word & 0b11 != 0b11 {
                Ok(word & 0xFFFF)
            } else {
                Ok(word)
            }
        } else {
            // Crosses page boundary — fall back to slow path
            let first = Mmu::fetch16(self, self.pc)?;
            if first & 0b11 != 0b11 {
                Ok(first as u32)
            } else {
                let second = Mmu::fetch16(self, self.pc + 2)?;
                Ok((first as u32) | ((second as u32) << 16))
            }
        }
    }

    /// Run up to `count` instructions in a tight loop.
    ///
    /// This eliminates the per-instruction overhead of the main loop:
    ///   - No `?` on step() (saves 11% = Result::branch)
    ///   - No main_clock increment (saves loop overhead)
    ///   - No WFI check on every instruction (only on WFI)
    ///   - No UART input check on every instruction
    ///
    /// Returns the number of instructions actually executed. Stops early
    /// on trap (returns count - remaining) or WFI (returns count - remaining).
    pub fn run_batch(&mut self, count: u64) -> u64 {
        // Sync PLIC state to mip before running. Interrupts may have
        // been triggered between batches (e.g. UART RX from keyboard
        // input). Without this, the guest stays stuck in WFI because
        // the PLIC evaluation inside the loop only runs every 1024
        // clocks — and if the guest is in WFI, the loop returns on the
        // first iteration before the PLIC is ever evaluated.
        let (meip, seip) = self.bus.plic.evaluate_interrupt();
        if meip { self.csr.mip |= 1 << 11; } else { self.csr.mip &= !(1 << 11); }
        if seip { self.csr.mip |= 1 << 9; } else { self.csr.mip &= !(1 << 9); }

        for i in 0..count {
            self.clock = self.clock.wrapping_add(1);

            // PLIC evaluation every 1024 clocks. mtime is advanced
            // by the main loop using wall-clock time (see main.rs).
            if self.clock & 0x3FF == 0 {
                let (meip, seip) = self.bus.plic.evaluate_interrupt();
                if meip { self.csr.mip |= 1 << 11; } else { self.csr.mip &= !(1 << 11); }
                if seip { self.csr.mip |= 1 << 9; } else { self.csr.mip &= !(1 << 9); }
            }

            // Interrupt check
            if (self.csr.mip & self.csr.mie) != 0 {
                self.wfi = false;
                if let Some(cause) = self.pending_interrupt() {
                    if self.handle_interrupt(cause).is_err() {
                        return i;
                    }
                    continue;
                }
            }

            // WFI check — only return if still sleeping
            if self.wfi {
                return i;
            }

            // Decode cache
            let cache_index = ((self.pc >> 2) as usize) & 0x3FFF;
            let (decoded, length) = if self.decode_cache[cache_index].valid && self.decode_cache[cache_index].pc == self.pc {
                let entry = &self.decode_cache[cache_index];
                (entry.decoded, entry.length)
            } else {
                let raw = match self.fetch() {
                    Ok(inst) => inst,
                    Err(trap) => {
                        self.fetch_page_valid = false;
                        self.data_read_valid = false;
                        self.data_write_valid = false;
                        if self.handle_trap(trap).is_err() {
                            return i;
                        }
                        continue;
                    }
                };
                let decoded = decode(raw);
                let length = decoded.length;
                self.decode_cache[cache_index] = DecodeEntry { pc: self.pc, valid: true, decoded, length };
                (decoded, length)
            };

            self.current_instruction_length = length;

            match execute::execute(decoded, self) {
                Ok(flow) => match flow {
                    ExecFlow::Next => {
                        self.pc = self.pc.wrapping_add(length as u64);
                    }
                    ExecFlow::Jump(addr) => {
                        self.pc = addr;
                    }
                },
                Err(trap) => {
                    self.fetch_page_valid = false;
                    self.data_read_valid = false;
                    self.data_write_valid = false;
                    if self.handle_trap(trap).is_err() {
                        return i;
                    }
                }
            }
        }
        count
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        self.clock = self.clock.wrapping_add(1);

        // PLIC evaluation every 1024 clocks. mtime is advanced by the
        // caller using wall-clock time (see main.rs / web.rs).
        if self.clock & 0x3FF == 0 || self.wfi {
            let (meip, seip) = self.bus.plic.evaluate_interrupt();
            if meip { self.csr.mip |= 1 << 11; } else { self.csr.mip &= !(1 << 11); }
            if seip { self.csr.mip |= 1 << 9; } else { self.csr.mip &= !(1 << 9); }
        }

        if (self.csr.mip & self.csr.mie) != 0 {
            // wakeup
            self.wfi = false;
        }

        if (self.csr.mip & self.csr.mie) != 0 {
            if let Some(cause) = self.pending_interrupt() {
                self.handle_interrupt(cause)?;
                return Ok(());
            }
        }

        // sleeping in idle don't do anything
        if self.wfi {
            return Ok(());
        }

        // Decode cache: 16384 entries (64KB of code coverage).
        // Hash: bits 2-15 of PC (4-byte aligned instructions).
        //
        // CRITICAL: do NOT copy the full DecodeEntry (~42 bytes) on every
        // instruction. The original code did `let entry = self.decode_cache[i]`
        // which copies the entire struct including the Instruction enum
        // (32+ bytes) on EVERY instruction — even on cache misses where
        // the copy is immediately discarded. At 30 MIPS that's 1.2 GB/s
        // of wasted memory copies.
        //
        // Instead, read only `pc` and `valid` (9 bytes) for the hit check,
        // and only copy the full entry on an actual hit.
        let cache_index = ((self.pc >> 2) as usize) & 0x3FFF;

        let (decoded, length) = if self.decode_cache[cache_index].valid && self.decode_cache[cache_index].pc == self.pc {
            // Hit: copy decoded instruction + length from the cache.
            let entry = &self.decode_cache[cache_index];
            (entry.decoded, entry.length)
        } else {
            let raw = match self.fetch() {
                Ok(inst) => inst,
                Err(trap) => {
                    #[cfg(debug_assertions)]
                    eprintln!("\n TRAP CAUGHT at PC 0x{:016x}: {:?}", self.pc, trap);

                    self.handle_trap(trap)?;
                    return Ok(());
                }
            };

            let decoded = decode(raw);
            let length = decoded.length;

            self.decode_cache[cache_index] = DecodeEntry { pc: self.pc, valid: true, decoded, length };

            (decoded, length)
        };

        self.current_instruction_length = length;

        match execute::execute(decoded, self) {
            Ok(flow) => match flow {
                ExecFlow::Next => {
                    self.pc = self.pc.wrapping_add(length as u64);
                }
                ExecFlow::Jump(target) => {
                    self.pc = target;
                }
            },
            Err(trap) => {
                self.handle_trap(trap)?;
            }
        }

        Ok(())
    }

    pub fn handle_interrupt(&mut self, cause: u64) -> Result<(), Trap> {
        // Invalidate fetch cache — privilege/PC change
        self.fetch_page_valid = false;
        self.data_read_valid = false;
        self.data_write_valid = false;

        // delegate to S-mode if enabled and we're not already in M-mode.
        let delegated = self.privilege_mode != PrivilegeMode::Machine && self.csr.is_interrupt_delegated(cause);

        if delegated {
            return self.handle_supervisor_interrupt(cause);
        }

        // save PC.
        self.csr.mepc = self.pc;

        // interrupt bit.
        self.csr.mcause = (1u64 << 63) | cause;

        // MPIE <- MIE
        let mie = (self.csr.mstatus >> 3) & 1;

        self.csr.mstatus &= !(1 << 7);
        self.csr.mstatus |= mie << 7;

        // MIE <- 0
        self.csr.mstatus &= !(1 << 3);

        // MPP <- previous privilege
        self.csr.mstatus &= !(0b11 << 11);

        let mpp = match self.privilege_mode {
            PrivilegeMode::User => 0,
            PrivilegeMode::Supervisor => 1,
            PrivilegeMode::Machine => 3,
        };

        self.csr.mstatus |= (mpp as u64) << 11;

        self.privilege_mode = PrivilegeMode::Machine;

        let base = self.csr.mtvec & !0b11;
        let mode = self.csr.mtvec & 0b11;

        self.pc = match mode {
            0 => base,
            1 => base + (cause << 2),
            _ => base,
        };

        Ok(())
    }
    fn handle_supervisor_interrupt(&mut self, cause: u64) -> Result<(), Trap> {
        self.csr.sepc = self.pc;

        self.csr.scause = (1u64 << 63) | cause;
        self.csr.stval = 0;

        // SPIE <- SIE
        let sie = (self.csr.mstatus >> 1) & 1;

        self.csr.mstatus &= !(1 << 5);
        self.csr.mstatus |= sie << 5;

        // SIE <- 0
        self.csr.mstatus &= !(1 << 1);

        // SPP <- previous privilege
        self.csr.mstatus &= !(1 << 8);

        if self.privilege_mode == PrivilegeMode::Supervisor {
            self.csr.mstatus |= 1 << 8;
        }

        self.privilege_mode = PrivilegeMode::Supervisor;

        let base = self.csr.stvec & !0b11;
        let mode = self.csr.stvec & 0b11;

        self.pc = match mode {
            0 => base,
            1 => base + (cause << 2),
            _ => base,
        };

        Ok(())
    }

    #[inline]
    pub fn pending_interrupt(&self) -> Option<u64> {
        // Pending and individually enabled interrupts
        let pending = self.csr.mip & self.csr.mie;
        if pending == 0 {
            return None;
        }

        // Global interrupt enables
        let m_enabled = self.privilege_mode != PrivilegeMode::Machine || ((self.csr.mstatus >> 3) & 1) != 0; // mstatus.MIE

        let s_enabled = self.privilege_mode == PrivilegeMode::User || (self.privilege_mode == PrivilegeMode::Supervisor && ((self.csr.mstatus >> 1) & 1) != 0); // mstatus.SIE

        // M-mode interrupts (bits NOT set in mideleg)
        let m_pending = pending & !self.csr.mideleg;
        if m_enabled && m_pending != 0 {
            if (m_pending & (1 << 11)) != 0 {
                return Some(11);
            } // Machine External
            if (m_pending & (1 << 3)) != 0 {
                return Some(3);
            } // Machine Software
            if (m_pending & (1 << 7)) != 0 {
                return Some(7);
            } // Machine Timer
        }

        // S-mode interrupts (bits SET in mideleg)
        let s_pending = pending & self.csr.mideleg;
        if s_enabled && s_pending != 0 {
            if (s_pending & (1 << 9)) != 0 {
                return Some(9);
            } // Supervisor External
            if (s_pending & (1 << 1)) != 0 {
                return Some(1);
            } // Supervisor Software
            if (s_pending & (1 << 5)) != 0 {
                return Some(5);
            } // Supervisor Timer
        }

        None
    }

    fn debug_print(&mut self) {
        self.clock += 1;
        if self.clock % 5_000_000 == 0 {
            let stip = (self.csr.mip >> 5) & 1;
            println!("HEARTBEAT: PC = {:#018x} | mtime = {:<20} | stimecmp = {:<20} | STIP = {}", self.pc, self.bus.clint.mtime, self.csr.stimecmp, stip);
        }
    }
}

// trap handler
impl Cpu {
    pub fn handle_trap(&mut self, trap: Trap) -> Result<(), Trap> {
        // Invalidate fetch cache — privilege/PC change
        self.fetch_page_valid = false;
        self.data_read_valid = false;
        self.data_write_valid = false;

        let cause = Self::exception_cause(&trap);

        let delegated = self.privilege_mode != PrivilegeMode::Machine && self.csr.is_exception_delegated(cause);

        if delegated {
            return self.handle_supervisor_trap(trap);
        }

        // save faulting PC.
        self.csr.mepc = self.pc;

        // clear mtval by default.
        self.csr.mtval = 0;

        // record trap cause.
        self.csr.mcause = match trap {
            Trap::InstructionAddressMisaligned(addr) => {
                self.csr.mtval = addr;
                0
            }

            Trap::InstructionAccessFault(addr) => {
                self.csr.mtval = addr;
                1
            }

            Trap::IllegalInstruction(inst) => {
                self.csr.mtval = inst as u64;
                2
            }

            Trap::Breakpoint => 3,

            Trap::LoadAddressMisaligned(addr) => {
                self.csr.mtval = addr;
                4
            }

            Trap::LoadAccessFault(addr) => {
                self.csr.mtval = addr;
                5
            }

            Trap::StoreAddressMisaligned(addr) => {
                self.csr.mtval = addr;
                6
            }

            Trap::StoreAccessFault(addr) => {
                self.csr.mtval = addr;
                7
            }

            Trap::EcallFromUMode => 8,
            Trap::EcallFromSMode => 9,
            Trap::EcallFromMMode => 11,

            Trap::InstructionPageFault(addr) => {
                self.csr.mtval = addr;
                12
            }

            Trap::LoadPageFault(addr) => {
                self.csr.mtval = addr;
                13
            }

            Trap::StorePageFault(addr) => {
                self.csr.mtval = addr;
                15
            }
        };

        // mstatus updates

        // MPIE <- MIE
        let mie = (self.csr.mstatus >> 3) & 1;

        self.csr.mstatus &= !(1 << 7);
        self.csr.mstatus |= mie << 7;

        // MIE <- 0
        self.csr.mstatus &= !(1 << 3);

        // MPP <- previous privilege mode
        self.csr.mstatus &= !(0b11 << 11);

        let mpp = match self.privilege_mode {
            PrivilegeMode::User => 0,
            PrivilegeMode::Supervisor => 1,
            PrivilegeMode::Machine => 3,
        };

        self.csr.mstatus |= (mpp as u64) << 11;

        // enter Machine mode.
        self.privilege_mode = PrivilegeMode::Machine;

        // Jump to trap vector.

        let base = self.csr.mtvec & !0b11;
        let mode = self.csr.mtvec & 0b11;

        self.pc = match mode {
            // Direct
            0 => base,

            // vectored
            // interrupts use BASE + 4*cause.
            // exceptions always go to BASE.
            1 => {
                let interrupt = (self.csr.mcause >> 63) != 0;

                if interrupt { base + ((self.csr.mcause & !(1 << 63)) << 2) } else { base }
            }

            // reserved modes
            _ => base,
        };

        #[cfg(debug_assertions)]
        {
            let is_interrupt = (self.csr.mcause >> 63) != 0;
            if self.csr.mcause != 9 && !is_interrupt {
                println!("Trap: mcause={} mepc={:#018x} mtval={:#018x}", self.csr.mcause, self.csr.mepc, self.csr.mtval);
            }
        }

        Ok(())
    }

    fn handle_supervisor_trap(&mut self, trap: Trap) -> Result<(), Trap> {
        // save faulting PC.
        self.csr.sepc = self.pc;

        // clear stval by default.
        self.csr.stval = 0;

        // record trap cause.
        self.csr.scause = match trap {
            Trap::InstructionAddressMisaligned(addr) => {
                self.csr.stval = addr;
                0
            }

            Trap::InstructionAccessFault(addr) => {
                self.csr.stval = addr;
                1
            }

            Trap::IllegalInstruction(inst) => {
                self.csr.stval = inst as u64;
                2
            }

            Trap::Breakpoint => 3,

            Trap::LoadAddressMisaligned(addr) => {
                self.csr.stval = addr;
                4
            }

            Trap::LoadAccessFault(addr) => {
                self.csr.stval = addr;
                5
            }

            Trap::StoreAddressMisaligned(addr) => {
                self.csr.stval = addr;
                6
            }

            Trap::StoreAccessFault(addr) => {
                self.csr.stval = addr;
                7
            }

            Trap::EcallFromUMode => 8,
            Trap::EcallFromSMode => 9,
            Trap::EcallFromMMode => 11,

            Trap::InstructionPageFault(addr) => {
                self.csr.stval = addr;
                12
            }

            Trap::LoadPageFault(addr) => {
                self.csr.stval = addr;
                13
            }

            Trap::StorePageFault(addr) => {
                self.csr.stval = addr;
                15
            }
        };

        // sstatus updates

        // SPIE <- SIE
        let sie = (self.csr.mstatus >> 1) & 1;
        self.csr.mstatus &= !(1 << 5);
        self.csr.mstatus |= sie << 5;

        self.csr.mstatus &= !(1 << 1);

        self.csr.mstatus &= !(1 << 8);
        if self.privilege_mode == PrivilegeMode::Supervisor {
            self.csr.mstatus |= 1 << 8;
        }

        // enter Supervisor mode.
        self.privilege_mode = PrivilegeMode::Supervisor;

        // Jump to stvec.
        let base = self.csr.stvec & !0b11;
        let mode = self.csr.stvec & 0b11;

        self.pc = match mode {
            0 => base,
            1 => {
                let interrupt = (self.csr.scause >> 63) != 0;
                if interrupt { base + ((self.csr.scause & !(1 << 63)) << 2) } else { base }
            }
            _ => base,
        };

        Ok(())
    }

    #[inline]
    fn exception_cause(trap: &Trap) -> u64 {
        match trap {
            Trap::InstructionAddressMisaligned(_) => 0,
            Trap::InstructionAccessFault(_) => 1,
            Trap::IllegalInstruction(_) => 2,
            Trap::Breakpoint => 3,
            Trap::LoadAddressMisaligned(_) => 4,
            Trap::LoadAccessFault(_) => 5,
            Trap::StoreAddressMisaligned(_) => 6,
            Trap::StoreAccessFault(_) => 7,
            Trap::EcallFromUMode => 8,
            Trap::EcallFromSMode => 9,
            Trap::EcallFromMMode => 11,
            Trap::InstructionPageFault(_) => 12,
            Trap::LoadPageFault(_) => 13,
            Trap::StorePageFault(_) => 15,
        }
    }
}

// reservation helpers
impl Cpu {
    #[inline]
    pub fn reserve_address(&mut self, addr: u64) {
        self.reservation = Some(addr);
    }

    #[inline]
    pub fn clear_reservation(&mut self) {
        self.reservation = None;
    }

    #[inline]
    pub fn reservation_matches(&self, addr: u64) -> bool {
        self.reservation == Some(addr)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
