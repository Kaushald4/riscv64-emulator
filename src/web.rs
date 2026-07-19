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
            // write safely via a raw pointer
            *ptr::addr_of_mut!(VM) = Some(cpu);
        }
    }

    pub fn run(max_cycles: u64) -> Result<u32, ()> {
        let cpu = unsafe { (*ptr::addr_of_mut!(VM)).as_mut().ok_or(())? };

        // Poll at the START of the batch — picks up any frames the JS side
        // delivered while we were sleeping (the 4ms timer between batches
        // is when most `netrx` postMessages arrive).
        cpu.bus.drain_all_virtio().ok();

        for i in 0..max_cycles {
            if cpu.step().is_err() {
                return Err(());
            }

            if i % 1_000 == 0 {
                if cpu.bus.uart.is_interrupting() {
                    cpu.bus.plic.trigger_interrupt(10);
                }
            }
        }

        
        cpu.bus.drain_all_virtio().ok();

        /*
            return a simple status flag to javaScript:
              0 = CPU is asleep (WFI)
              1 = CPU is awake and processing
        */
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
pub unsafe extern "C" fn glasshart_run(max_cycles: u32) -> i32 {
    match wasm_web::run(max_cycles as u64) {
        // 1 (i am awake) or 0 (i am asleep)
        Ok(status) => status as i32,
        // -1 (i am trapped)
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
