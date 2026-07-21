#[cfg(target_arch = "wasm32")]
mod wasm_web {
    use crate::cpu::Cpu;
    use crate::cpu::register::Reg;
    use std::ptr;

    const RAM_BASE: u64 = 0x8000_0000;
    const DTB_ADDR: u64 = RAM_BASE + 0x2200000;
    const KERNEL_ADDR: u64 = 0x8020_0000;

    // static mut, perfectly safe for single-threaded WASM
    static mut VM: Option<Cpu> = None;

    pub fn boot(firmware: &[u8], kernel: &[u8], dtb: &[u8], rootfs: &[u8]) {
        let mut cpu = Cpu::new();

        cpu.bus.load_external_image(RAM_BASE, firmware).unwrap();
        cpu.bus.load_external_image(KERNEL_ADDR, kernel).unwrap();
        cpu.bus.load_external_image(DTB_ADDR, dtb).unwrap();
        cpu.bus.load_block_image(rootfs.to_vec());

        cpu.pc = RAM_BASE;
        cpu.regs.write(Reg::new(10), 0);
        cpu.regs.write(Reg::new(11), DTB_ADDR);

        unsafe {
            *ptr::addr_of_mut!(VM) = Some(cpu);
        }
    }

    /// Run up to `max_cycles` instructions. `elapsed_ms` is the wall-clock
    /// time since the last call, used to advance mtime (10MHz CLINT =
    /// 10,000 ticks/ms). This keeps the guest timer at real-time rate
    /// regardless of emulator speed.
    pub fn run(max_cycles: u64, elapsed_ms: f64) -> Result<u32, ()> {
        let cpu = unsafe { (*ptr::addr_of_mut!(VM)).as_mut().ok_or(())? };

        cpu.bus.drain_all_virtio().ok();

        if elapsed_ms > 0.0 {
            let elapsed_ticks = (elapsed_ms * 10_000.0) as u64;
            cpu.bus.clint.mtime = cpu.bus.clint.mtime.wrapping_add(elapsed_ticks);
            cpu.csr.time = cpu.bus.clint.mtime;
            if cpu.bus.clint.mtime >= cpu.bus.clint.mtimecmp {
                cpu.csr.mip |= 1 << 7;
            }
            if cpu.bus.clint.mtime >= cpu.csr.stimecmp {
                cpu.csr.mip |= 1 << 5;
            }
        }

        // Run in batches using run_batch(). Between batches, check UART
        // and drain virtio. Return early (status=2) when UART output is
        // pending so the JS scheduler can drain it to the terminal —
        // this keeps typing latency low (<3ms) even with large batches.
        let mut remaining = max_cycles;
        while remaining > 0 {
            let batch = remaining.min(50_000);
            let executed = cpu.run_batch(batch);
            remaining = remaining.saturating_sub(executed);

            if cpu.bus.uart.is_interrupting() {
                cpu.bus.plic.trigger_interrupt(10);
            }
            cpu.bus.drain_all_virtio().ok();

            // Early return when UART has output pending — lets the JS
            // side drain it to the terminal promptly for low typing latency.
            if cpu.bus.uart.has_tx_output() {
                return Ok(2);
            }

            if cpu.wfi {
                break;
            }
        }

        if cpu.wfi { Ok(0) } else { Ok(1) }
    }

    pub fn uart_read() -> Option<u8> {
        let cpu = unsafe { (*ptr::addr_of_mut!(VM)).as_mut()? };
        cpu.bus.uart.pop_tx()
    }

    pub fn uart_write(byte: u8) {
        if let Some(cpu) = unsafe { (*ptr::addr_of_mut!(VM)).as_mut() } {
            let should_interrupt = cpu.bus.uart.push_rx(byte);
            if should_interrupt {
                cpu.bus.plic.trigger_interrupt(10);
            }
        }
    }

    pub fn net_stats() -> (u64, u64, u64) {
        let cpu = unsafe { (*ptr::addr_of_mut!(VM)).as_mut() };
        match cpu {
            Some(c) => (
                c.bus.virtio_net.driver_features,
                c.bus.virtio_net_dev.gro_frames_in,
                c.bus.virtio_net_dev.gro_segments_out,
            ),
            None => (0, 0, 0),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_alloc(size: usize) -> *mut u8 {
    let mut buf: Vec<u8> = vec![0; size];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_boot(fw_ptr: *const u8, fw_len: usize, k_ptr: *const u8, k_len: usize, dtb_ptr: *const u8, dtb_len: usize, rfs_ptr: *const u8, rfs_len: usize) {
    let fw = std::slice::from_raw_parts(fw_ptr, fw_len);
    let k = std::slice::from_raw_parts(k_ptr, k_len);
    let dtb = std::slice::from_raw_parts(dtb_ptr, dtb_len);
    let rfs = std::slice::from_raw_parts(rfs_ptr, rfs_len);
    wasm_web::boot(fw, k, dtb, rfs);
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_run(max_cycles: u32, elapsed_ms: f64) -> i32 {
    match wasm_web::run(max_cycles as u64, elapsed_ms) {
        Ok(status) => status as i32,
        Err(_) => -1,
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_uart_read() -> u32 {
    match wasm_web::uart_read() {
        Some(b) => 0x100 | (b as u32),
        None => 0,
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_uart_write(byte: u32) {
    wasm_web::uart_write(byte as u8);
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glasshart_net_stats() -> u64 {
    let (df, frames_in, segs_out) = wasm_web::net_stats();
    (df << 32) | ((segs_out & 0xffff) << 16) | (frames_in & 0xffff)
}
