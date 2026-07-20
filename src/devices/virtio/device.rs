use crate::{cpu::memory::Memory, devices::plic::Plic, trap::Trap};

use super::descriptor::VirtqDesc;

pub struct VirtioContext<'a> {
    pub memory: &'a mut Memory,
    pub plic: &'a mut Plic,
    pub interrupt_status: &'a mut u32,
    pub irq: u32,
    /// feature bits the guest driver has negotiated. this lets the
    /// device decide whether to use optional features like GSO/GRO
    /// (only if the guest advertised support via VIRTIO_NET_F_GUEST_TSO4).
    pub driver_features: u64,
}

pub trait VirtIODevice {
    fn device_id(&self) -> u32;
    fn host_features(&self) -> u64;
    fn read_config32(&self, offset: u64) -> u32;
    fn write_config32(&mut self, _offset: u64, _value: u32) {}
    fn process_descriptor_chain(&mut self, ctx: &mut VirtioContext, desc_chain: &[VirtqDesc], queue_idx: u16) -> Result<u32, Trap>;
}
