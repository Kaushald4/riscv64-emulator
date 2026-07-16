use crate::{cpu::PrivilegeMode, trap::Trap};
use std::collections::HashMap;

const MISA_MXL_RV64: u64 = 2 << 62;
const MISA_A: u64 = 1 << ('A' as u8 - b'A');
const MISA_C: u64 = 1 << ('C' as u8 - b'A');
const MISA_D: u64 = 1 << ('D' as u8 - b'A');
const MISA_F: u64 = 1 << ('F' as u8 - b'A');
const MISA_I: u64 = 1 << ('I' as u8 - b'A');
const MISA_M: u64 = 1 << ('M' as u8 - b'A');
const MISA_S: u64 = 1 << ('S' as u8 - b'A');
const MISA_U: u64 = 1 << ('U' as u8 - b'A');
const MISA: u64 = MISA_MXL_RV64 | MISA_I | MISA_M | MISA_A | MISA_F | MISA_D | MISA_C | MISA_S | MISA_U;

pub const MSTATUS_MIE: u64 = 1 << 3;
pub const MSTATUS_MPIE: u64 = 1 << 7;

pub const SSTATUS_MASK: u64 = 0x8000_0003_000D_E162;

// supervisor bits
pub const MSTATUS_SIE: u64 = 1 << 1;
pub const MSTATUS_SPIE: u64 = 1 << 5;
pub const MSTATUS_SPP: u64 = 1 << 8;

pub const MSTATUS_MPP_SHIFT: u64 = 11;
pub const MSTATUS_MPP_MASK: u64 = 0b11 << MSTATUS_MPP_SHIFT;

// machine information registers
pub const CSR_MVENDORID: u16 = 0xF11;
pub const CSR_MARCHID: u16 = 0xF12;
pub const CSR_MIMPID: u16 = 0xF13;
pub const CSR_MHARTID: u16 = 0xF14;

// machine trap setup
pub const CSR_MSTATUS: u16 = 0x300;
pub const CSR_MISA: u16 = 0x301;
pub const CSR_MEDELEG: u16 = 0x302;
pub const CSR_MIDELEG: u16 = 0x303;
pub const CSR_MIE: u16 = 0x304;
pub const CSR_MTVEC: u16 = 0x305;

// machine trap handling
pub const CSR_MSCRATCH: u16 = 0x340;
pub const CSR_MEPC: u16 = 0x341;
pub const CSR_MCAUSE: u16 = 0x342;
pub const CSR_MTVAL: u16 = 0x343;
pub const CSR_MIP: u16 = 0x344;

pub const CSR_CYCLE: u16 = 0xC00;
pub const CSR_TIME: u16 = 0xC01;

// supervisor
pub const CSR_SATP: u16 = 0x180;
pub const CSR_SSTATUS: u16 = 0x100;
pub const CSR_SIE: u16 = 0x104;
pub const CSR_STVEC: u16 = 0x105;
pub const CSR_SSCRATCH: u16 = 0x140;
pub const CSR_SEPC: u16 = 0x141;
pub const CSR_SCAUSE: u16 = 0x142;
pub const CSR_STVAL: u16 = 0x143;
pub const CSR_SIP: u16 = 0x144;
pub const SSIP: u64 = 1 << 1;
pub const STIP: u64 = 1 << 5;
pub const SEIP: u64 = 1 << 9;

pub const SIE_MASK: u64 = SSIP | STIP | SEIP;
pub const SIP_MASK: u64 = SSIP | STIP | SEIP;

// floats csr
pub const CSR_FFLAGS: u16 = 0x001;
pub const CSR_FRM: u16 = 0x002;
pub const CSR_FCSR: u16 = 0x003;

// optional
pub const CSR_MNSTATUS: u16 = 0x744;

pub struct Csr {
    // machine information
    mhartid: u64,

    // frequently used CSRs
    pub mstatus: u64,
    pub misa: u64,
    pub medeleg: u64,
    pub mideleg: u64,
    pub mie: u64,
    pub mtvec: u64,

    pub mscratch: u64,
    pub mepc: u64,
    pub mcause: u64,
    pub mtval: u64,
    pub mip: u64,

    // supervisor
    pub satp: u64,
    pub sepc: u64,
    pub scause: u64,
    pub stval: u64,
    pub sscratch: u64,
    pub stvec: u64,

    // PMP
    pub pmpcfg: [u64; 16],
    pub pmpaddr: [u64; 64],

    // floats csr
    pub fflags: u64,
    pub frm: u64,
    pub fcsr: u64,

    pub time: u64,
    pub cycle: u64,

    // everything else
    extra: HashMap<u16, u64>,
}

impl Csr {
    pub fn new() -> Self {
        Self {
            mhartid: 0,

            mstatus: 0,
            misa: MISA,
            medeleg: 0,
            mideleg: 0,
            mie: 0,
            mtvec: 0,

            mscratch: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            mip: 0,

            satp: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            sscratch: 0,
            stvec: 0,

            pmpcfg: [0; 16],
            pmpaddr: [0; 64],

            fflags: 0,
            frm: 0,
            fcsr: 0,

            time: 0,
            cycle: 0,

            extra: HashMap::new(),
        }
    }

    #[inline]
    pub fn read(&self, csr: u16) -> Result<u64, Trap> {
        match csr {
            CSR_MHARTID => Ok(self.mhartid),

            CSR_MSTATUS => Ok(self.mstatus),
            CSR_MISA => Ok(self.misa),
            CSR_MEDELEG => Ok(self.medeleg),
            CSR_MIDELEG => Ok(self.mideleg),

            CSR_MTVEC => Ok(self.mtvec),

            CSR_MSCRATCH => Ok(self.mscratch),
            CSR_MEPC => Ok(self.mepc),
            CSR_MCAUSE => Ok(self.mcause),
            CSR_MTVAL => Ok(self.mtval),
            CSR_MIP => Ok(self.mip),
            CSR_MIE => Ok(self.mie),

            CSR_SATP => Ok(self.satp),

            // supervisor
            CSR_SSTATUS => Ok(self.mstatus & SSTATUS_MASK),
            CSR_STVEC => Ok(self.stvec),
            CSR_SEPC => Ok(self.sepc),
            CSR_SCAUSE => Ok(self.scause),
            CSR_STVAL => Ok(self.stval),

            CSR_SSCRATCH => Ok(self.sscratch),
            CSR_SIE => Ok(self.mie & SIE_MASK),
            CSR_SIP => Ok(self.mip & SIP_MASK),

            // floats
            CSR_FFLAGS => Ok(self.fflags),
            CSR_FRM => Ok(self.frm),
            CSR_FCSR => Ok(self.fcsr),

            CSR_TIME => Ok(self.time),
            CSR_CYCLE => Ok(self.cycle),

            0x3A0..=0x3AF => Ok(self.pmpcfg[(csr - 0x3A0) as usize]),

            0x3B0..=0x3EF => Ok(self.pmpaddr[(csr - 0x3B0) as usize]),

            _ => Ok(*self.extra.get(&csr).unwrap_or(&0)),
        }
    }

    #[inline]
    pub fn write(&mut self, csr: u16, value: u64) -> Result<(), Trap> {
        match csr {
            CSR_MSTATUS => self.mstatus = value,
            CSR_MISA => self.misa = value,
            CSR_MEDELEG => self.medeleg = value,
            CSR_MIDELEG => self.mideleg = value,
            CSR_MTVEC => self.mtvec = value,

            CSR_MSCRATCH => self.mscratch = value,
            CSR_MEPC => self.mepc = value,
            CSR_MCAUSE => self.mcause = value,
            CSR_MTVAL => self.mtval = value,
            CSR_MIP => self.mip = value,
            CSR_MIE => self.mie = value,

            CSR_SATP => self.satp = value,

            // supervisor
            // Replace the old CSR_SSTATUS line with this:
            CSR_SSTATUS => {
                self.mstatus = (self.mstatus & !SSTATUS_MASK) | (value & SSTATUS_MASK);
            }
            CSR_STVEC => self.stvec = value,
            CSR_SEPC => self.sepc = value,
            CSR_SCAUSE => self.scause = value,
            CSR_STVAL => self.stval = value,
            CSR_SIE => {
                self.mie = (self.mie & !SIE_MASK) | (value & SIE_MASK);
            }

            CSR_SIP => {
                self.mip = (self.mip & !SIP_MASK) | (value & SIP_MASK);
            }

            // floats
            CSR_FFLAGS => {
                self.fflags = value & 0x1f;
                self.fcsr = (self.frm << 5) | self.fflags;
            }

            CSR_FRM => {
                self.frm = value & 0x7;
                self.fcsr = (self.frm << 5) | self.fflags;
            }

            CSR_FCSR => {
                self.fcsr = value & 0xff;
                self.fflags = self.fcsr & 0x1f;
                self.frm = (self.fcsr >> 5) & 0x7;
            }

            0x3A0..=0x3AF => {
                self.pmpcfg[(csr - 0x3A0) as usize] = value;
            }

            0x3B0..=0x3EF => {
                self.pmpaddr[(csr - 0x3B0) as usize] = value;
            }

            CSR_MHARTID | CSR_MVENDORID | CSR_MARCHID | CSR_MIMPID => {
                return Err(Trap::IllegalInstruction(csr as u32));
            }

            _ => {
                // Store any other writable CSR.
                self.extra.insert(csr, value);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn sum(&self) -> bool {
        ((self.mstatus >> 18) & 1) != 0
    }

    #[inline]
    pub fn mxr(&self) -> bool {
        ((self.mstatus >> 19) & 1) != 0
    }

    #[inline]
    pub fn mprv(&self) -> bool {
        ((self.mstatus >> 17) & 1) != 0
    }

    #[inline]
    pub fn mpp(&self) -> PrivilegeMode {
        match (self.mstatus >> 11) & 0b11 {
            0 => PrivilegeMode::User,
            1 => PrivilegeMode::Supervisor,
            3 => PrivilegeMode::Machine,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn is_exception_delegated(&self, cause: u64) -> bool {
        ((self.medeleg >> cause) & 1) != 0
    }

    #[inline]
    pub fn is_interrupt_delegated(&self, cause: u64) -> bool {
        ((self.mideleg >> cause) & 1) != 0
    }
}

impl Default for Csr {
    fn default() -> Self {
        Self::new()
    }
}
