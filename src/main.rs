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
        let addr = RAM_BASE + i as u64;

        if let Err(e) = cpu.bus.write8(addr, *byte) {
            println!("Failed at address {:#018x}", addr);
            return Err(e);
        }
    }

    let dtb = fs::read("firmware/virt.dtb").expect("failed to read firmware/virt.dtb");

    for (i, byte) in dtb.iter().enumerate() {
        if let Err(e) = cpu.bus.write8(DTB_ADDR + i as u64, *byte) {
            println!("Failed at address {:#018x}", DTB_ADDR + i as u64);
            return Err(e);
        };
    }

    let kernel = fs::read("kernel/kernel_6.6").expect("failed to read kernel/kernel_6.6");

    for (i, byte) in kernel.iter().enumerate() {
        if let Err(e) = cpu.bus.write8(KERNEL_ADDR + i as u64, *byte) {
            println!("Failed at address {:#018x}", DTB_ADDR + i as u64);
            return Err(e);
        }
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
