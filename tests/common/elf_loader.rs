use std::{fs, path::Path};

use elf::{ElfBytes, endian::AnyEndian};

const SHF_EXECINSTR: u64 = 0x4;

pub struct Elf {
    instructions: Vec<u32>,
}

impl Elf {
    pub fn instructions(&self) -> impl Iterator<Item = u32> + '_ {
        self.instructions.iter().copied()
    }
}

pub fn load_elf(path: impl AsRef<Path>) -> Elf {
    let bytes = fs::read(path).expect("failed to read ELF");

    let elf = ElfBytes::<AnyEndian>::minimal_parse(&bytes).expect("invalid ELF");

    let (shdrs, _) = elf.section_headers_with_strtab().expect("failed to read section headers");

    let shdrs = shdrs.expect("ELF has no section headers");

    let mut instructions = Vec::new();

    for shdr in shdrs.iter() {
        if (shdr.sh_flags & SHF_EXECINSTR) == 0 {
            continue;
        }

        let (data, _) = elf.section_data(&shdr).expect("failed to read executable section");

        for chunk in data.chunks_exact(4) {
            instructions.push(u32::from_le_bytes(chunk.try_into().unwrap()));
        }
    }

    Elf { instructions }
}
