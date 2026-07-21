#![cfg_attr(target_arch = "wasm32", no_main)]

#[cfg(not(target_arch = "wasm32"))]
mod native_main {
    use std::{fs, io::Read, sync::mpsc, thread};

    use crossterm::terminal;
    use glasshart_emulator::{
        cpu::{Cpu, register::Reg},
        trap::Trap,
    };

    const RAM_BASE: u64 = 0x8000_0000;
    const DTB_ADDR: u64 = RAM_BASE + 0x2200000;
    const KERNEL_ADDR: u64 = 0x8020_0000;

    pub fn run() -> Result<(), Trap> {
        terminal::enable_raw_mode().unwrap();

        let (tx, rx) = mpsc::channel::<u8>();

        thread::spawn(move || {
            let mut stdin = std::io::stdin();
            let mut buffer = [0u8; 1];

            while stdin.read_exact(&mut buffer).is_ok() {
                if buffer[0] == 0x03 {
                    let _ = terminal::disable_raw_mode();
                    std::process::exit(0);
                }

                let _ = tx.send(buffer[0]);
            }
        });

        let mut cpu = Cpu::new();

        let firmware = fs::read("firmware/fw_jump.bin").expect("failed to read firmware/fw_jump.bin");
        cpu.bus.load_external_image(RAM_BASE, &firmware)?;

        let dtb = fs::read("firmware/virt.dtb").expect("failed to read firmware/virt.dtb");
        cpu.bus.load_external_image(DTB_ADDR, &dtb)?;

        let kernel = fs::read("kernel/kernel_6.6").expect("failed to read kernel/kernel_6.6");
        cpu.bus.load_external_image(KERNEL_ADDR, &kernel)?;

        cpu.pc = RAM_BASE;

        cpu.regs.write(Reg::new(10), 0);
        cpu.regs.write(Reg::new(11), DTB_ADDR);

        print!("Booting GlassHart VM...\r\n");

        // Advance mtime based on wall-clock time. The CLINT runs at 10MHz,
        // so 1 tick = 100ns. This makes the guest's timer fire at the
        // correct rate (250Hz for CONFIG_HZ=250), regardless of how fast
        // or slow the emulator runs.
        //
        // Previously, mtime was advanced by 128 per 128 instructions inside
        // run_batch. At 30 MIPS that's 30M ticks/sec — 3x too fast for a
        // 10MHz CLINT. The kernel's scheduler got confused by the wrong
        // time accounting and put dd to sleep (WFI) 94% of the time,
        // giving only 5.8% CPU utilization and 11.8 MB/s throughput.
        let mut last_tick = std::time::Instant::now();
        let mut wfi_spin = 0u32;
        loop {
            // Advance mtime by wall-clock elapsed time (10MHz CLINT).
            let now = std::time::Instant::now();
            let elapsed_ns = now.duration_since(last_tick).as_nanos() as u64;
            if elapsed_ns >= 100 {
                let elapsed_ticks = elapsed_ns / 100;
                cpu.bus.clint.mtime = cpu.bus.clint.mtime.wrapping_add(elapsed_ticks);
                cpu.csr.time = cpu.bus.clint.mtime;
                if cpu.bus.clint.mtime >= cpu.bus.clint.mtimecmp {
                    cpu.csr.mip |= 1 << 7;
                }
                if cpu.bus.clint.mtime >= cpu.csr.stimecmp {
                    cpu.csr.mip |= 1 << 5;
                }
                last_tick = now;
            }

            // Check UART input.
            while let Ok(byte) = rx.try_recv() {
                let should_interrupt = cpu.bus.uart.push_rx(byte);
                if should_interrupt {
                    cpu.bus.plic.trigger_interrupt(10);
                }
            }
            if cpu.bus.uart.is_interrupting() {
                cpu.bus.plic.trigger_interrupt(10);
            }

            let _executed = cpu.run_batch(100_000);

            if cpu.wfi {
                wfi_spin += 1;
                // Only sleep after a very long spin (truly idle, e.g. shell
                // prompt with no input). Don't advance mtime here — the
                // wall-clock code above handles that. The sleep is just to
                // avoid burning 100% CPU when the guest is doing nothing.
                if wfi_spin >= 10_000_000 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    wfi_spin = 0;
                }
            } else {
                wfi_spin = 0;
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), glasshart_emulator::trap::Trap> {
    native_main::run()
}
