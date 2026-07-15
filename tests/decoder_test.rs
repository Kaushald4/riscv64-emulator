mod common;

use common::elf_loader::load_elf;

use glasshart_emulator::decode::decode;
use glasshart_emulator::instruction::Instruction;

use std::fs;

fn decode_suite(prefix: &str) {
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

        println!("Testing {name}");

        let elf = load_elf(&path);

        for (i, raw) in elf.instructions().enumerate() {
            let inst = decode(raw);

            println!("{:04}  0x{:08x}  {:?}", i, raw, inst);

            if matches!(inst.instruction, Instruction::Undefined { .. }) {
                panic!("{}: instruction #{} (0x{:08x}) failed to decode", name, i, raw);
            }
        }
    }
}

#[test]
fn decode_rv64ui() {
    decode_suite("rv64ui-p-");
}

#[test]
fn decode_rv64um() {
    decode_suite("rv64um-p-");
}

#[test]
fn decode_rv64ua() {
    decode_suite("rv64ua-p-");
}

#[test]
fn decode_rv64uc() {
    decode_suite("rv64uc-p-");
}

#[test]
fn decode_rv64uf() {
    decode_suite("rv64uf-p-");
}

#[test]
fn decode_rv64ud() {
    decode_suite("rv64ud-p-");
}
