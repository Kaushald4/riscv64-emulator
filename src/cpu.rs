pub mod bus;
pub mod csr;
pub mod execute;
pub mod f_register;
pub mod memory;
pub mod register;

use crate::{
    cpu::{bus::Bus, csr::Csr},
    decode::decode,
    devices::{
        Device,
        virtio::{descriptor::VirtqDesc, request::VirtIOBlkReqHeader},
    },
    mmu::Mmu,
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
pub enum PrivilegeMode {
    Machine,
    Supervisor,
    User,
}

pub struct Cpu {
    pub regs: Register,
    pub f_regs: FRegister,
    pub pc: u64,
    pub bus: Bus,
    pub current_instruction_length: u8,
    pub privilege_mode: PrivilegeMode,
    pub csr: Csr,
    // reservation for LR/SC
    pub reservation: Option<u64>,

    pub inst_count: u64,
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
            reservation: None,

            inst_count: 0,
        }
    }

    fn fetch(&mut self) -> Result<u32, Trap> {
        if self.pc & 1 != 0 {
            return Err(Trap::InstructionAddressMisaligned(self.pc));
        }

        let first = Mmu::read16(self, self.pc)?;

        if first & 0b11 != 0b11 {
            Ok(first as u32)
        } else {
            // let second = self.bus.read16(self.pc + 2)?;
            let second = Mmu::read16(self, self.pc + 2)?;

            Ok((first as u32) | ((second as u32) << 16))
        }
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        self.bus.tick();

        if let Some(queue) = self.bus.virtio.take_queue_notify() {
            self.process_virtio_queue(queue)?;
        }

        #[cfg(debug_assertions)]
        self.debug_print();

        self.csr.time = self.bus.clint.mtime;

        if self.bus.clint.mtime >= self.bus.clint.mtimecmp {
            self.csr.mip |= 1 << 7;
        } else {
            self.csr.mip &= !(1 << 7);
        }

        if self.bus.clint.mtime >= self.csr.stimecmp {
            // set STIP
            self.csr.mip |= 1 << 5;
        } else {
            // clear STIP
            self.csr.mip &= !(1 << 5);
        }

        // software interrupt (MSIP)
        if (self.bus.clint.msip & 1) != 0 {
            // set MSIP
            self.csr.mip |= 1 << 3;
        } else {
            // clear MSIP
            self.csr.mip &= !(1 << 3);
        }

        // evaluate external interrupts from PLIC
        let (meip, seip) = self.bus.plic.evaluate_interrupt();

        // machine external interrupt pending (Bit 11)
        if meip {
            self.csr.mip |= 1 << 11;
        } else {
            self.csr.mip &= !(1 << 11);
        }

        // supervisor external interrupt pending (Bit 9)
        if seip {
            self.csr.mip |= 1 << 9;
        } else {
            self.csr.mip &= !(1 << 9);
        }

        if let Some(cause) = self.pending_interrupt() {
            self.handle_interrupt(cause)?;
            return Ok(());
        }

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

    fn process_virtio_queue(&mut self, queue: u16) -> Result<(), Trap> {
        let q_idx = queue as usize;
        let ready = self.bus.virtio.queues[q_idx].ready;

        if !ready {
            return Ok(());
        }

        let avail_ring = self.bus.virtio.queues[q_idx].avail_ring;
        let size = self.bus.virtio.queues[q_idx].size;
        let mut triggered = false;

        loop {
            let last_avail_idx = self.bus.virtio.queues[q_idx].last_avail_idx;
            let avail_idx = self.bus.read16(avail_ring + 2)?;

            if last_avail_idx == avail_idx {
                break;
            }

            let ring_entry = avail_ring + 4 + ((last_avail_idx as u64 % size as u64) * 2);
            let desc_index = self.bus.read16(ring_entry)?;

            self.bus.virtio.queues[q_idx].last_avail_idx = last_avail_idx.wrapping_add(1);

            let desc_table = self.bus.virtio.queues[q_idx].desc_table;

            // Read the 3-part descriptor chain (Header -> Data -> Status)
            let desc0 = self.read_virtq_desc(desc_table + (desc_index as u64 * 16))?;
            let desc1 = self.read_virtq_desc(desc_table + (desc0.next as u64 * 16))?;
            let desc2 = self.read_virtq_desc(desc_table + (desc1.next as u64 * 16))?;

            let header = self.read_blk_req_header(desc0.addr)?;
            let mut written_len = 0;

            match header.request_type {
                0 => {
                    // read (Disk -> RAM)
                    let length = desc1.len as usize;
                    let start = header.sector as usize * 512;

                    // TODO: Improve this for performance
                    // read byte-by-byte (temp only to pass the borrow checking error)
                    for i in 0..length {
                        let byte = self.bus.virtio.device.image[start + i];
                        self.bus.write8(desc1.addr + i as u64, byte)?;
                    }

                    self.bus.write8(desc2.addr, 0)?;
                    written_len = length as u32 + 1;
                }
                1 => {
                    // write (RAM -> Disk)
                    let length = desc1.len as usize;
                    let start = header.sector as usize * 512;

                    for i in 0..length {
                        let byte = self.bus.read8(desc1.addr + i as u64)?;
                        self.bus.virtio.device.image[start + i] = byte;
                    }

                    self.bus.write8(desc2.addr, 0)?;
                    written_len = 1;
                }
                _ => {
                    self.bus.write8(desc2.addr, 2)?;
                    written_len = 1;
                }
            }

            // populate the USED ring
            let used_ring = self.bus.virtio.queues[q_idx].used_ring;
            let used_idx = self.bus.read16(used_ring + 2)?;
            let used_elem_addr = used_ring + 4 + ((used_idx as u64 % size as u64) * 8);

            self.bus.write32(used_elem_addr, desc_index as u32)?;
            self.bus.write32(used_elem_addr + 4, written_len)?;

            // advance the used_idx
            self.bus.write16(used_ring + 2, used_idx.wrapping_add(1))?;

            triggered = true;
        }

        // only fire the interrupt once we have processed everything in the batch!
        if triggered {
            self.bus.virtio.interrupt_status |= 0x1;
            self.bus.plic.trigger_interrupt(1);
        }

        Ok(())
    }

    fn read_virtq_desc(&mut self, addr: u64) -> Result<VirtqDesc, Trap> {
        Ok(VirtqDesc {
            addr: self.bus.read64(addr)?,
            len: self.bus.read32(addr + 8)?,
            flags: self.bus.read16(addr + 12)?,
            next: self.bus.read16(addr + 14)?,
        })
    }
    fn read_blk_req_header(&mut self, addr: u64) -> Result<VirtIOBlkReqHeader, Trap> {
        Ok(VirtIOBlkReqHeader {
            request_type: self.bus.read32(addr)?,
            reserved: self.bus.read32(addr + 4)?,
            sector: self.bus.read64(addr + 8)?,
        })
    }

    pub fn handle_interrupt(&mut self, cause: u64) -> Result<(), Trap> {
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
        self.inst_count += 1;
        if self.inst_count % 5_000_000 == 0 {
            let stip = (self.csr.mip >> 5) & 1;
            println!("HEARTBEAT: PC = {:#018x} | mtime = {:<20} | stimecmp = {:<20} | STIP = {}", self.pc, self.bus.clint.mtime, self.csr.stimecmp, stip);
        }
    }
}

// trap handler
impl Cpu {
    pub fn handle_trap(&mut self, trap: Trap) -> Result<(), Trap> {
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
