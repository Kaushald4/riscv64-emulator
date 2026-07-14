mod common;

use common::elf_loader::load_elf;

use glasshart_emulator::decode::decode;
use glasshart_emulator::instruction::Instruction;

#[test]
fn decode_all_rv64ui() {
    use std::fs;

    let dir = "tests/riscv-tests";

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let name = path.file_name().unwrap().to_string_lossy();

        if !name.starts_with("rv64um-p-") {
            continue;
        }
        if path.extension().is_some() {
            continue;
        }

        println!("Testing {name}");

        let elf = load_elf(&path);

        for (i, raw) in elf.instructions().enumerate() {
            let inst = decode(raw);

            println!("{:04}  0x{:08x}  {:?}", i, raw, inst);

            if matches!(inst, Instruction::Undefined { .. }) {
                panic!("{}: instruction #{} (0x{:08x}) failed to decode", name, i, raw);
            }
        }
    }
}
