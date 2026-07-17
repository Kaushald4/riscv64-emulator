use std::{fs, io::Read, sync::mpsc, thread};

use crossterm::terminal;
use glasshart_emulator::{
    cpu::{Cpu, register::Reg},
    trap::Trap,
};

const RAM_BASE: u64 = 0x8000_0000;
const DTB_ADDR: u64 = RAM_BASE + 0x2200000;
const KERNEL_ADDR: u64 = 0x8020_0000;

fn main() -> Result<(), Trap> {
    // terminal::enable_raw_mode().unwrap();

    let (tx, rx) = mpsc::channel::<u8>();

    thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buffer = [0u8; 1];

        while stdin.read_exact(&mut buffer).is_ok() {
            // Ctrl+C
            if buffer[0] == 0x03 {
                std::process::exit(0);
            }

            let _ = tx.send(buffer[0]);
        }
    });

    let mut cpu = Cpu::new();

    let firmware = fs::read("firmware/fw_jump.bin").expect("failed to read firmware/fw_jump.bin");

    for (i, byte) in firmware.iter().enumerate() {
        cpu.bus.write8(RAM_BASE + i as u64, *byte)?;
    }

    let dtb = fs::read("firmware/virt.dtb").expect("failed to read firmware/virt.dtb");

    for (i, byte) in dtb.iter().enumerate() {
        cpu.bus.write8(DTB_ADDR + i as u64, *byte)?;
    }

    let kernel = fs::read("kernel/kernel_6.6").expect("failed to read kernel/kernel_6.6");

    for (i, byte) in kernel.iter().enumerate() {
        cpu.bus.write8(KERNEL_ADDR + i as u64, *byte)?;
    }

    cpu.pc = RAM_BASE;

    // a0 = hartid
    cpu.regs.write(Reg::new(10), 0);

    // a1 = DTB address
    cpu.regs.write(Reg::new(11), DTB_ADDR);

    println!("Booting OpenSBI...");
    let mut main_clock = 0u64;
    loop {
        main_clock = main_clock.wrapping_add(1);

        if main_clock % 10_000 == 0 {
            if let Ok(byte) = rx.try_recv() {
                let should_interrupt = cpu.bus.uart.push_rx(byte);
                if should_interrupt {
                    cpu.bus.plic.trigger_interrupt(10);
                }
            }

            if cpu.bus.uart.is_interrupting() {
                cpu.bus.plic.trigger_interrupt(10);
            }
        }
        cpu.step()?;
    }
}
