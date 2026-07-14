mod common;

use std::fs;

use common::elf_runner::load_execution_elf;

use glasshart_emulator::cpu::Cpu;

fn execute_suite(prefix: &str) {
    let dir = "tests/riscv-tests";

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let name = path.file_name().unwrap().to_string_lossy();

        if !name.starts_with(prefix) {
            continue;
        }

        if path.extension().is_some() {
            continue;
        }

        println!("Running {name}");

        let elf = load_execution_elf(&path);

        let mut cpu = Cpu::new();

        elf.load_into(&mut cpu.bus).unwrap();

        cpu.pc = elf.entry();

        // address of the "tohost" symbol in this ELF.
        let tohost = elf.tohost().expect("ELF has no tohost symbol");

        let mut instructions = 0usize;

        loop {
            if let Err(e) = cpu.step() {
                panic!("{} failed after {} instructions\nPC = {:#018x}\n{:?}", name, instructions, cpu.pc, e);
            }

            // read the current tohost value directly from guest memory.
            let value = cpu.bus.read32(tohost).unwrap() as u64;

            if value != 0 {
                if value == 1 {
                    println!("{name}: PASS");
                } else {
                    panic!("{name}: FAIL test {}", value >> 1);
                }

                break;
            }

            instructions += 1;

            if instructions > 10_000_000 {
                panic!("{name}: exceeded instruction limit");
            }
        }
    }
}

#[test]
fn execute_rv64ui() {
    execute_suite("rv64ui-p-");
}
