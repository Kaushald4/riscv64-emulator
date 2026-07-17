pub mod clint;
pub mod plic;
pub mod uart;
pub mod virtio;

pub trait Device {
    fn tick(&mut self);
}
