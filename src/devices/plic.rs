use crate::trap::Trap;

pub const PLIC_BASE: u64 = 0x0C00_0000;
pub const PLIC_SIZE: u64 = 0x0400_0000;

#[derive(Debug)]
pub struct Plic {
    // IRQ 0 is reserved. IRQs 1-255 are usable.
    pub priority: [u32; 256],
    // 8 * 32 = 256 bits
    pub pending: [u32; 8],

    // Context 0 = M-mode (Machine)
    // Context 1 = S-mode (Supervisor)
    pub enable_m: [u32; 8],
    pub enable_s: [u32; 8],

    pub threshold_m: u32,
    pub threshold_s: u32,
}

impl Plic {
    pub fn new() -> Self {
        Self {
            priority: [0; 256],
            pending: [0; 8],
            enable_m: [0; 8],
            enable_s: [0; 8],
            threshold_m: 0,
            threshold_s: 0,
        }
    }

    /// called by VirtIO/UART to trigger an interrupt
    pub fn trigger_interrupt(&mut self, irq: u32) {
        if irq > 0 && irq < 256 {
            let index = (irq / 32) as usize;
            let bit = irq % 32;
            self.pending[index] |= 1 << bit;
        }
    }

    /// called by the CPU on every tick to evaluate if MEIP or SEIP should be asserted
    pub fn evaluate_interrupt(&self) -> (bool, bool) {
        let mut meip = false;
        let mut seip = false;

        for i in 1..256 {
            let index = (i / 32) as usize;
            let bit = i % 32;

            // if the interrupt is pending...
            if (self.pending[index] & (1 << bit)) != 0 {
                let prio = self.priority[i as usize];

                // check Context 0 (M-mode)
                if (self.enable_m[index] & (1 << bit)) != 0 && prio > self.threshold_m {
                    meip = true;
                }

                // check Context 1 (S-mode)
                if (self.enable_s[index] & (1 << bit)) != 0 && prio > self.threshold_s {
                    seip = true;
                }
            }
        }

        (meip, seip)
    }

    /// returns the highest priority pending IRQ and clears its pending bit.
    fn claim(&mut self, context: u32) -> u32 {
        let enable = if context == 0 { &self.enable_m } else { &self.enable_s };
        let threshold = if context == 0 { self.threshold_m } else { self.threshold_s };

        let mut max_irq = 0;
        let mut max_prio = 0;

        for i in 1..256 {
            let index = (i / 32) as usize;
            let bit = i % 32;

            if (self.pending[index] & (1 << bit)) != 0 && (enable[index] & (1 << bit)) != 0 {
                let prio = self.priority[i as usize];
                if prio > threshold && prio > max_prio {
                    max_prio = prio;
                    max_irq = i;
                }
            }
        }

        // when linux reads the claim register, we MUST clear the pending bit
        if max_irq != 0 {
            let index = (max_irq / 32) as usize;
            let bit = max_irq % 32;
            self.pending[index] &= !(1 << bit);
        }

        max_irq
    }

    #[inline]
    fn offset(addr: u64) -> Result<u64, Trap> {
        if !(PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return Err(Trap::LoadAccessFault(addr));
        }
        Ok(addr - PLIC_BASE)
    }

    pub fn read32(&mut self, addr: u64) -> Result<u32, Trap> {
        let offset = Self::offset(addr)?;

        match offset {
            // interrupt priorities
            0x000000..=0x000FFF => {
                let irq = (offset / 4) as usize;
                if irq < 256 { Ok(self.priority[irq]) } else { Ok(0) }
            }
            // pending bits
            0x001000..=0x00107F => {
                let index = ((offset - 0x001000) / 4) as usize;
                if index < 8 { Ok(self.pending[index]) } else { Ok(0) }
            }
            // Context 0 (M-mode) enables
            0x002000..=0x00207F => {
                let index = ((offset - 0x002000) / 4) as usize;
                if index < 8 { Ok(self.enable_m[index]) } else { Ok(0) }
            }
            // Context 1 (S-mode) enables
            0x002080..=0x0020FF => {
                let index = ((offset - 0x002080) / 4) as usize;
                if index < 8 { Ok(self.enable_s[index]) } else { Ok(0) }
            }
            // Context 0 (M-mode) threshold and Claim
            0x200000 => Ok(self.threshold_m),
            0x200004 => Ok(self.claim(0)),

            // Context 1 (S-mode) Threshold and Claim
            0x201000 => Ok(self.threshold_s),
            0x201004 => Ok(self.claim(1)),

            _ => Ok(0),
        }
    }

    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        let offset = Self::offset(addr)?;

        match offset {
            0x000000..=0x000FFF => {
                let irq = (offset / 4) as usize;
                if irq < 256 {
                    self.priority[irq] = value;
                }
            }
            0x001000..=0x00107F => {
                let index = ((offset - 0x001000) / 4) as usize;
                if index < 8 {
                    self.pending[index] = value;
                }
            }
            0x002000..=0x00207F => {
                let index = ((offset - 0x002000) / 4) as usize;
                if index < 8 {
                    self.enable_m[index] = value;
                }
            }
            0x002080..=0x0020FF => {
                let index = ((offset - 0x002080) / 4) as usize;
                if index < 8 {
                    self.enable_s[index] = value;
                }
            }

            0x200000 => self.threshold_m = value,
            0x200004 => { /* linux writes back to complete, no action strictly needed in emu */ }

            0x201000 => self.threshold_s = value,
            0x201004 => { /* linux writes back to complete, no action strictly needed in emu */ }

            _ => {}
        }
        Ok(())
    }

    // helper wrappers to convert 8/16/64 bit accesses to 32-bit accesses.
    // PLIC is strictly 32-bit aligned hardware.
    pub fn read8(&mut self, addr: u64) -> Result<u8, Trap> {
        let val32 = self.read32(addr & !0b11)?;
        let shift = (addr & 0b11) * 8;
        Ok(((val32 >> shift) & 0xFF) as u8)
    }

    pub fn read16(&mut self, addr: u64) -> Result<u16, Trap> {
        let val32 = self.read32(addr & !0b11)?;
        let shift = (addr & 0b11) * 8;
        Ok(((val32 >> shift) & 0xFFFF) as u16)
    }

    pub fn read64(&mut self, addr: u64) -> Result<u64, Trap> {
        let lo = self.read32(addr)? as u64;
        let hi = self.read32(addr + 4)? as u64;
        Ok((hi << 32) | lo)
    }

    pub fn write8(&mut self, _addr: u64, _value: u8) -> Result<(), Trap> {
        Ok(())
    }
    pub fn write16(&mut self, _addr: u64, _value: u16) -> Result<(), Trap> {
        Ok(())
    }

    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        self.write32(addr, (value & 0xFFFFFFFF) as u32)?;
        self.write32(addr + 4, (value >> 32) as u32)?;
        Ok(())
    }
}
