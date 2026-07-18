pub mod cpu;
pub mod decode;
pub mod devices;
pub mod instruction;
pub mod mmu;
pub mod opcode;
pub mod trap;

#[cfg(target_arch = "wasm32")]
mod web;
