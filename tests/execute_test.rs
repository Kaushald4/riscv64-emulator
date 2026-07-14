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

        let mut instructions = 0usize;

        loop {
            if let Err(e) = cpu.step() {
                panic!("{} failed after {} instructions\nPC = {:#018x}\n{:?}", name, instructions, cpu.pc, e);
            }

            if let Some(value) = cpu.bus.tohost {
                let code = value >> 1;

                if code == 1 {
                    println!("{name}: PASS");
                } else {
                    panic!("{name}: FAIL test {}", code);
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
