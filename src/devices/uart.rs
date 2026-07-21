use crate::trap::Trap;
use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};

pub const UART_BASE: u64 = 0x1000_0000;
pub const UART_SIZE: u64 = 0x100;

#[derive(Debug, Default)]
pub struct Uart {
    dll: u8,
    dlm: u8,
    ier: u8,
    fcr: u8,
    lcr: u8,
    mcr: u8,
    pub lsr: u8,
    msr: u8,
    scr: u8,
    thr: u8,
    rbr: VecDeque<u8>,
    tx_queue: VecDeque<u8>,
    tx_int_pending: bool,
}

impl Uart {
    pub fn new() -> Self {
        Self {
            lsr: 0x60,
            rbr: VecDeque::new(),
            tx_queue: VecDeque::new(),
            tx_int_pending: false,
            ..Default::default()
        }
    }

    pub fn is_interrupting(&self) -> bool {
        // RX Interrupt - data is in the queue, and RX interrupts are enabled
        let rx_pending = (self.ier & 0x01) != 0 && (self.lsr & 0x01) != 0;

        // TX Interrupt - The flag is set
        let tx_pending = self.tx_int_pending;

        rx_pending || tx_pending
    }

    pub fn push_rx(&mut self, byte: u8) -> bool {
        self.rbr.push_back(byte);
        self.lsr |= 0x01;
        (self.ier & 0x01) != 0
    }

    /// Queue TX output for WASM console rendering.
    pub fn pop_tx(&mut self) -> Option<u8> {
        self.tx_queue.pop_front()
    }

    /// Check if there are pending TX bytes for the terminal.
    pub fn has_tx_output(&self) -> bool {
        !self.tx_queue.is_empty()
    }

    #[inline]
    fn offset(addr: u64) -> Result<u64, Trap> {
        if !(UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return Err(Trap::LoadAccessFault(addr));
        }
        Ok(addr - UART_BASE)
    }

    pub fn read8(&mut self, addr: u64) -> Result<u8, Trap> {
        let off = Self::offset(addr)?;

        let value = match off {
            0 => {
                if self.lcr & 0x80 != 0 {
                    self.dll
                } else {
                    // pop the oldest keystroke from the front of the queue
                    let data = self.rbr.pop_front().unwrap_or(0);

                    // if the queue is now empty, clear the data ready bit
                    if self.rbr.is_empty() {
                        self.lsr &= !0x01;
                    }

                    data
                }
            }
            1 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm
                } else {
                    self.ier
                }
            }
            2 => {
                if (self.ier & 0x01) != 0 && (self.lsr & 0x01) != 0 {
                    // RX Interrupt has highest priority
                    0x04
                } else if self.tx_int_pending {
                    self.tx_int_pending = false;
                    // TX Interrupt
                    0x02
                } else {
                    // no interrupt pending
                    0x01
                }
            }
            3 => self.lcr,
            4 => self.mcr,
            5 => self.lsr,
            6 => self.msr,
            7 => self.scr,
            _ => 0,
        };
        Ok(value)
    }

    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        let off = Self::offset(addr)?;

        match off {
            0 => {
                if self.lcr & 0x80 != 0 {
                    self.dll = value;
                } else {
                    self.thr = value;
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        print!("{}", value as char);
                        io::stdout().flush().unwrap();
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.tx_queue.push_back(value);
                    }

                    if (self.ier & 0x02) != 0 {
                        self.tx_int_pending = true;
                    }
                }
            }
            1 => {
                if self.lcr & 0x80 != 0 {
                    self.dlm = value;
                } else {
                    // track if linux is turning TX interrupts on or off
                    let tx_enable_old = (self.ier & 0x02) != 0;
                    self.ier = value;
                    let tx_enable_new = (self.ier & 0x02) != 0;

                    if !tx_enable_old && tx_enable_new {
                        self.tx_int_pending = true;
                    } else if !tx_enable_new {
                        self.tx_int_pending = false;
                    }
                }
            }
            2 => self.fcr = value,
            3 => self.lcr = value,
            4 => self.mcr = value,
            7 => self.scr = value,
            _ => {}
        }
        Ok(())
    }

    pub fn read16(&mut self, addr: u64) -> Result<u16, Trap> {
        Ok(self.read8(addr)? as u16 | ((self.read8(addr + 1)? as u16) << 8))
    }

    pub fn read32(&mut self, addr: u64) -> Result<u32, Trap> {
        let mut value = 0u32;
        for i in 0..4 {
            value |= (self.read8(addr + i)? as u32) << (i * 8);
        }
        Ok(value)
    }

    pub fn read64(&mut self, addr: u64) -> Result<u64, Trap> {
        let mut value = 0u64;
        for i in 0..8 {
            value |= (self.read8(addr + i)? as u64) << (i * 8);
        }
        Ok(value)
    }

    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        for i in 0..2 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }
        Ok(())
    }

    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        for i in 0..4 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }
        Ok(())
    }

    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        for i in 0..8 {
            self.write8(addr + i, ((value >> (i * 8)) & 0xff) as u8)?;
        }
        Ok(())
    }
}
