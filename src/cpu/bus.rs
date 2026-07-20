use crate::{
    cpu::memory::Memory,
    devices::{
        Device,
        clint::{CLINT_BASE, CLINT_SIZE, Clint},
        plic::{PLIC_BASE, PLIC_SIZE, Plic},
        uart::{UART_BASE, UART_SIZE, Uart},
        virtio::{
            block::device::VirtIOBlock,
            device::VirtioContext,
            mmio::VirtIOMmio,
            net::{
                backend::{NetworkBackend, TapBackend},
                device::VirtIONet,
            },
            transport,
        },
    },
    trap::Trap,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::devices::virtio::block::backend::FileBackend;

#[cfg(target_arch = "wasm32")]
use crate::devices::virtio::block::backend::MemoryBackend;

#[cfg(not(target_arch = "wasm32"))]
type VirtIOBlockDefault = VirtIOBlock<FileBackend>;

#[cfg(target_arch = "wasm32")]
type VirtIOBlockDefault = VirtIOBlock<MemoryBackend>;

pub const RAM_BASE: u64 = 0x8000_0000;
pub const VIRTIO_BASE: u64 = 0x1000_1000;
pub const VIRTIO_NET_BASE: u64 = 0x1000_2000;
pub const VIRTIO_SIZE: u64 = 0x1000;

/// PLIC IRQ lines. These must match the device tree (`examples/virt.dtb`):
///   virtio_mmio@10001000 (block) → interrupts = 1
///   virtio_mmio@10002000 (net)   → interrupts = 3
pub const VIRTIO_BLOCK_IRQ: u32 = 1;
pub const VIRTIO_NET_IRQ: u32 = 3;

#[derive(Debug, Clone, Copy)]
pub enum MisalignedAccess {
    Trap,
    Emulate,
}

pub struct Bus {
    pub ram: Memory,
    misaligned: MisalignedAccess,
    pub clint: Clint,
    pub uart: Uart,
    pub plic: Plic,
    pub virtio: VirtIOMmio,
    pub virtio_net: VirtIOMmio,
    pub virtio_block: VirtIOBlockDefault,
    pub virtio_net_dev: VirtIONet,
}

impl Device for Bus {
    fn tick(&mut self) {
        self.clint.tick();
    }
}

impl Bus {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(ram_size: usize) -> Self {
        Self {
            ram: Memory::new(ram_size),
            misaligned: MisalignedAccess::Emulate,
            clint: Clint::new(),
            uart: Uart::new(),
            plic: Plic::new(),
            virtio: VirtIOMmio::new(),
            virtio_net: VirtIOMmio::with_queues(2),
            virtio_block: VirtIOBlockDefault::new(FileBackend::new("kernel/base.img")),
            virtio_net_dev: {
                VirtIONet::new(
                    TapBackend::new("tap0")
                        .map(|t| Box::new(t) as Box<dyn NetworkBackend>)
                        .unwrap_or_else(|e| {
                            eprintln!("virtio-net: TAP unavailable ({}), falling back to no-op", e);
                            Box::new(crate::devices::virtio::net::backend::DummyBackend::new())
                        }),
                )
            },
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(ram_size: usize) -> Self {
        use crate::devices::virtio::net::backend::WebRtcBackend;
        Self {
            ram: Memory::new(ram_size),
            misaligned: MisalignedAccess::Emulate,
            clint: Clint::new(),
            uart: Uart::new(),
            plic: Plic::new(),
            virtio: VirtIOMmio::new(),
            virtio_net: VirtIOMmio::with_queues(2),
            virtio_block: VirtIOBlockDefault::new(MemoryBackend::new(524288)),
            virtio_net_dev: VirtIONet::new(Box::new(WebRtcBackend::new())),
        }
    }

    /// Load an external blob at the given physical address — used by the
    /// WASM host to inject firmware, DTB, and kernel images at runtime.
    pub fn load_external_image(&mut self, addr: u64, data: &[u8]) -> Result<(), Trap> {
        for (i, byte) in data.iter().enumerate() {
            self.write8(addr + i as u64, *byte)?;
        }
        Ok(())
    }

    /// Replace the virtio block disk image at runtime (WASM host provides
    /// the raw disk bytes).
    #[cfg(target_arch = "wasm32")]
    pub fn load_block_image(&mut self, data: Vec<u8>) {
        self.virtio_block = VirtIOBlockDefault::new(MemoryBackend::from_bytes(data));
    }

    /// Drain all pending virtio work: block I/O, net TX (doorbell-driven), and
    /// net RX (polled every tick so frames the host pushes via WebRTC make it
    /// into the guest's RX buffers).
    pub fn drain_all_virtio(&mut self) -> Result<(), Trap> {
        self.drain_virtio_block()?;
        self.drain_virtio_net_tx()?;
        self.drain_virtio_net_rx()
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

    fn virtio_block_read32(&self, offset: u64) -> u32 {
        use crate::devices::virtio::device::VirtIODevice;
        self.virtio.read32(self.virtio_block.device_id(), self.virtio_block.host_features(), |off| self.virtio_block.read_config32(off), offset)
    }

    fn virtio_net_read32(&self, offset: u64) -> u32 {
        use crate::devices::virtio::device::VirtIODevice;
        self.virtio_net.read32(self.virtio_net_dev.device_id(), self.virtio_net_dev.host_features(), |off| self.virtio_net_dev.read_config32(off), offset)
    }

    fn virtio_offset(&self, addr: u64, base: u64) -> Option<u64> {
        if (base..base + VIRTIO_SIZE).contains(&addr) { Some(addr - base) } else { None }
    }

    fn drain_virtio_block(&mut self) -> Result<(), Trap> {
        let Bus { ram, plic, virtio, virtio_block, .. } = self;

        while let Some(queue_idx) = virtio.take_queue_notify() {
            let queue = &mut virtio.queues[queue_idx as usize];

            if !queue.ready {
                continue;
            }

            let mut ctx = VirtioContext {
                memory: ram,
                plic,
                interrupt_status: &mut virtio.interrupt_status,
                irq: VIRTIO_BLOCK_IRQ,
                driver_features: virtio.driver_features,
            };
            transport::drain_queue(virtio_block, &mut ctx, queue, queue_idx)?;
        }

        Ok(())
    }

    fn drain_virtio_net_tx(&mut self) -> Result<(), Trap> {
        let Bus { ram, plic, virtio_net, virtio_net_dev, .. } = self;

        while let Some(queue_idx) = virtio_net.take_queue_notify() {
            // RX queue (0) is never notified by the driver — only TX.
            // Skip RX to avoid interrupt storms.
            if queue_idx == 0 {
                continue;
            }

            let queue = &mut virtio_net.queues[queue_idx as usize];

            if !queue.ready {
                continue;
            }

            let mut ctx = VirtioContext {
                memory: ram,
                plic,
                interrupt_status: &mut virtio_net.interrupt_status,
                irq: VIRTIO_NET_IRQ,
                driver_features: virtio_net.driver_features,
            };
            transport::drain_queue(virtio_net_dev, &mut ctx, queue, queue_idx)?;
        }

        Ok(())
    }

    /// Poll the network backend for inbound frames and deliver any that are
    /// waiting into driver-provided RX buffers. This is what makes the guest
    /// "see" packets arriving from the outside world (e.g. over WebRTC).
    ///
    /// Runs on every `drain_all_virtio` tick — the driver doesn't kick the RX
    /// queue, so we have to pull. Interrupts are gated per virtio 1.1 spec
    /// (NO_INTERRUPT flag + used_event) so a burst of RX frames produces
    /// at most one IRQ.
    fn drain_virtio_net_rx(&mut self) -> Result<(), Trap> {
        let Bus { ram, plic, virtio_net, virtio_net_dev, .. } = self;

        let interrupt_status = &mut virtio_net.interrupt_status;
        let rx_queue = &mut virtio_net.queues[0];

        let mut ctx = VirtioContext {
            memory: ram,
            plic,
            interrupt_status,
            irq: VIRTIO_NET_IRQ,
            driver_features: virtio_net.driver_features,
        };

        let triggered = virtio_net_dev.drain_rx(&mut ctx, rx_queue)?;

        if triggered {
            // Apply the same interrupt gating as drain_queue — respect the
            // guest's NO_INTERRUPT flag and used_event hint. Without this,
            // the RX path would fire an interrupt on every batch even when
            // the guest is busy-polling the used ring.
            let new_used_idx = ctx.memory.read16(rx_queue.used_ring - RAM_BASE + 2)?;
            if transport::should_interrupt(ctx.memory, rx_queue, new_used_idx)? {
                *ctx.interrupt_status |= 0x1;
                ctx.plic.trigger_interrupt(VIRTIO_NET_IRQ);
            }
        }

        Ok(())
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
        // narrow reads on virtio MMIO: allow only for config space
        if let Some(off) = self.virtio_offset(addr, VIRTIO_BASE) {
            if off >= 0x100 {
                let val = self.virtio_block_read32(off & !3);
                return Ok((val >> ((addr & 3) * 8)) as u8);
            }
            return Err(Trap::LoadAccessFault(addr));
        }
        if let Some(off) = self.virtio_offset(addr, VIRTIO_NET_BASE) {
            if off >= 0x100 {
                let val = self.virtio_net_read32(off & !3);
                return Ok((val >> ((addr & 3) * 8)) as u8);
            }
            return Err(Trap::LoadAccessFault(addr));
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
        // Narrow reads on virtio MMIO: allow only for config space
        if let Some(off) = self.virtio_offset(addr, VIRTIO_BASE) {
            if off >= 0x100 {
                let val = self.virtio_block_read32(off & !3);
                return Ok((val >> ((addr & 3) * 8)) as u16);
            }
            return Err(Trap::LoadAccessFault(addr));
        }
        if let Some(off) = self.virtio_offset(addr, VIRTIO_NET_BASE) {
            if off >= 0x100 {
                let val = self.virtio_net_read32(off & !3);
                return Ok((val >> ((addr & 3) * 8)) as u16);
            }
            return Err(Trap::LoadAccessFault(addr));
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
            return Ok(self.virtio_block_read32(addr - VIRTIO_BASE));
        }
        if (VIRTIO_NET_BASE..VIRTIO_NET_BASE + VIRTIO_SIZE).contains(&addr) {
            return Ok(self.virtio_net_read32(addr - VIRTIO_NET_BASE));
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
            self.drain_virtio_block()?;
            return Ok(());
        }
        if (VIRTIO_NET_BASE..VIRTIO_NET_BASE + VIRTIO_SIZE).contains(&addr) {
            self.virtio_net.write32(addr - VIRTIO_NET_BASE, value);
            self.drain_virtio_net_tx()?;
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

    pub fn read_dma(&self, addr: u64, buffer: &mut [u8]) -> Result<(), Trap> {
        if addr >= RAM_BASE { self.ram.read_bulk(addr - RAM_BASE, buffer) } else { Err(Trap::LoadAccessFault(addr)) }
    }

    pub fn write_dma(&mut self, addr: u64, data: &[u8]) -> Result<(), Trap> {
        if addr >= RAM_BASE { self.ram.load(addr - RAM_BASE, data) } else { Err(Trap::StoreAccessFault(addr)) }
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
