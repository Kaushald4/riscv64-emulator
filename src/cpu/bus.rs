use crate::{
    cpu::memory::Memory,
    devices::{
        Device,
        clint::{CLINT_BASE, CLINT_SIZE, Clint},
        plic::{PLIC_BASE, PLIC_SIZE, Plic},
        uart::{UART_BASE, UART_SIZE, Uart},
        virtio::mmio::VirtIOMmio,
    },
    trap::Trap,
};

pub const RAM_BASE: u64 = 0x8000_0000;
pub const VIRTIO_BASE: u64 = 0x1000_1000;
pub const VIRTIO_SIZE: u64 = 0x1000;

#[derive(Debug, Clone, Copy)]
pub enum MisalignedAccess {
    Trap,
    Emulate,
}

pub struct Bus {
    ram: Memory,
    misaligned: MisalignedAccess,
    pub clint: Clint,
    pub uart: Uart,
    pub plic: Plic,
    pub virtio: VirtIOMmio,
}

impl Device for Bus {
    fn tick(&mut self) {
        self.clint.tick();
    }
}

impl Bus {
    pub fn new(ram_size: usize) -> Self {
        Self {
            ram: Memory::new(ram_size),
            misaligned: MisalignedAccess::Emulate,
            clint: Clint::new(),
            uart: Uart::new(),
            plic: Plic::new(),
            virtio: VirtIOMmio::new(),
        }
    }

    #[inline]
    fn ram_offset(&self, addr: u64, trap: Trap) -> Result<u64, Trap> {
        if addr < RAM_BASE {
            return Err(trap);
        }

        let offset = addr - RAM_BASE;

        if offset >= self.ram.size() as u64 {
            return Err(trap);
        }

        Ok(offset)
    }

    // reads
    #[inline(always)]
    pub fn read8(&mut self, addr: u64) -> Result<u8, Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::LoadAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 3 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }
                return self.ram.read8(offset);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.read8(addr);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.read8(addr);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.read8(addr);
        }

        let offset = self.ram_offset(addr, Trap::LoadAccessFault(addr))?;
        self.ram.read8(offset)
    }

    #[inline(always)]
    pub fn read16(&mut self, addr: u64) -> Result<u16, Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::LoadAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 3 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }
                return self.ram.read16(offset);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.read16(addr);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.read16(addr);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.read16(addr);
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 3 != 0 {
            return self.read16_emulated(addr);
        }

        Err(Trap::LoadAccessFault(addr))
    }

    #[inline(always)]
    pub fn read32(&mut self, addr: u64) -> Result<u32, Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::LoadAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 3 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }
                return self.ram.read32(offset);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.read32(addr);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.read32(addr);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.read32(addr);
        }
        if (VIRTIO_BASE..VIRTIO_BASE + VIRTIO_SIZE).contains(&addr) {
            return Ok(self.virtio.read32(addr - VIRTIO_BASE));
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 3 != 0 {
            return self.read32_emulated(addr);
        }

        Err(Trap::LoadAccessFault(addr))
    }

    #[inline(always)]
    pub fn read64(&mut self, addr: u64) -> Result<u64, Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::LoadAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 3 != 0 {
                    return Err(Trap::LoadAddressMisaligned(addr));
                }
                return self.ram.read64(offset);
            }
        }
        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.read64(addr);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.read64(addr);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.read64(addr);
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 3 != 0 {
            return self.read64_emulated(addr);
        }

        Err(Trap::LoadAccessFault(addr))
    }

    // writes
    #[inline(always)]
    pub fn write8(&mut self, addr: u64, value: u8) -> Result<(), Trap> {
        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.write8(addr, value);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.write8(addr, value);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.write8(addr, value);
        }

        let offset = self.ram_offset(addr, Trap::StoreAccessFault(addr))?;
        self.ram.write8(offset, value)
    }

    #[inline(always)]
    pub fn write16(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::StoreAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 1 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                return self.ram.write16(offset, value);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.write16(addr, value);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.write16(addr, value);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.write16(addr, value);
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 1 != 0 {
            return self.write16_emulated(addr, value);
        }

        Err(Trap::StoreAccessFault(addr))
    }

    #[inline(always)]
    pub fn write32(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::StoreAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 1 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                return self.ram.write32(offset, value);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.write32(addr, value);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.write32(addr, value);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.write32(addr, value);
        }
        if (VIRTIO_BASE..VIRTIO_BASE + VIRTIO_SIZE).contains(&addr) {
            self.virtio.write32(addr - VIRTIO_BASE, value);
            return Ok(());
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 1 != 0 {
            return self.write32_emulated(addr, value);
        }

        Err(Trap::StoreAccessFault(addr))
    }

    #[inline(always)]
    pub fn write64(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        if addr >= RAM_BASE {
            if let Ok(offset) = self.ram_offset(addr, Trap::StoreAccessFault(addr)) {
                if matches!(self.misaligned, MisalignedAccess::Trap) && addr & 1 != 0 {
                    return Err(Trap::StoreAddressMisaligned(addr));
                }

                return self.ram.write64(offset, value);
            }
        }

        if (CLINT_BASE..CLINT_BASE + CLINT_SIZE).contains(&addr) {
            return self.clint.write64(addr, value);
        }
        if (UART_BASE..UART_BASE + UART_SIZE).contains(&addr) {
            return self.uart.write64(addr, value);
        }
        if (PLIC_BASE..PLIC_BASE + PLIC_SIZE).contains(&addr) {
            return self.plic.write64(addr, value);
        }

        if matches!(self.misaligned, MisalignedAccess::Emulate) && addr & 1 != 0 {
            return self.write64_emulated(addr, value);
        }

        Err(Trap::StoreAccessFault(addr))
    }

    pub fn load(&mut self, addr: u64, bytes: &[u8]) -> Result<(), Trap> {
        let offset = self.ram_offset(addr, Trap::StoreAccessFault(addr))?;
        self.ram.load(offset, bytes)
    }

    // private emulation helpers
    #[inline]
    fn read16_emulated(&mut self, addr: u64) -> Result<u16, Trap> {
        Ok((self.read8(addr)? as u16) | ((self.read8(addr + 1)? as u16) << 8))
    }

    #[inline]
    fn read32_emulated(&mut self, addr: u64) -> Result<u32, Trap> {
        let mut value = 0u32;

        for i in 0..4 {
            value |= (self.read8(addr + i)? as u32) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    fn read64_emulated(&mut self, addr: u64) -> Result<u64, Trap> {
        let mut value = 0u64;

        for i in 0..8 {
            value |= (self.read8(addr + i)? as u64) << (i * 8);
        }

        Ok(value)
    }

    #[inline]
    fn write16_emulated(&mut self, addr: u64, value: u16) -> Result<(), Trap> {
        self.write8(addr, value as u8)?;
        self.write8(addr + 1, (value >> 8) as u8)
    }

    #[inline]
    fn write32_emulated(&mut self, addr: u64, value: u32) -> Result<(), Trap> {
        for i in 0..4 {
            self.write8(addr + i, (value >> (i * 8)) as u8)?;
        }

        Ok(())
    }

    #[inline]
    fn write64_emulated(&mut self, addr: u64, value: u64) -> Result<(), Trap> {
        for i in 0..8 {
            self.write8(addr + i, (value >> (i * 8)) as u8)?;
        }

        Ok(())
    }
}
