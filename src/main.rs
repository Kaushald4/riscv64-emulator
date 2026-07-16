use std::fs;

use glasshart_emulator::{
    cpu::{Cpu, register::Reg},
    trap::Trap,
};

const RAM_BASE: u64 = 0x8000_0000;
const DTB_ADDR: u64 = 0x8000_0000 + 0x2200000;
const KERNEL_ADDR: u64 = 0x80200000;

fn main() -> Result<(), Trap> {
    let mut cpu = Cpu::new();

    let firmware = fs::read("firmware/fw_jump.bin").expect("failed to read firmware/fw_jump.bin");
    for (i, byte) in firmware.iter().enumerate() {
        cpu.bus.write8(RAM_BASE + i as u64, *byte).unwrap();
    }

    let kernel: Vec<u8> = fs::read("/home/kaushal/linux/arch/riscv/boot/Image").unwrap();
    println!("Kernel Size: {} MB", kernel.len() / 1024 / 1024);

    let kernel_end = KERNEL_ADDR + kernel.len() as u64;
    if kernel_end >= DTB_ADDR {
        panic!("FATAL: Linux Image is too large and will overwrite the DTB! Kernel ends at {:#x}, DTB starts at {:#x}", kernel_end, DTB_ADDR);
    }

    for (i, byte) in kernel.iter().enumerate() {
        cpu.bus.write8(KERNEL_ADDR + i as u64, *byte).unwrap();
    }

    let dtb = fs::read("firmware/virt.dtb").expect("failed to read firmware/virt.dtb");
    for (i, byte) in dtb.iter().enumerate() {
        cpu.bus.write8(DTB_ADDR + i as u64, *byte).unwrap();
    }

    cpu.pc = RAM_BASE;

    // a0 = hartid
    cpu.regs.write(Reg::new(10), 0);

    // a1 = DTB address
    cpu.regs.write(Reg::new(11), DTB_ADDR);

    println!("Booting OpenSBI...");

    loop {
        cpu.step()?;
    }
}
