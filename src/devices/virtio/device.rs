use crate::{cpu::memory::Memory, devices::plic::Plic, trap::Trap};

use super::queue::VirtQueue;

pub trait VirtIODevice {
    fn device_id(&self) -> u32;
    fn host_features(&self) -> u64;
    fn read_config32(&self, offset: u64) -> u32;
    fn write_config32(&mut self, _offset: u64, _value: u32) {}
    fn process_queue(&mut self, mem: &mut Memory, queue: &mut VirtQueue, plic: &mut Plic, interrupt_status: &mut u32) -> Result<(), Trap>;
}
