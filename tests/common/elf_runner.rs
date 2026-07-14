use std::{fs, path::Path};

use elf::{ElfBytes, abi::PT_LOAD, endian::AnyEndian};

use glasshart_emulator::{cpu::bus::Bus, trap::Trap};

pub struct LoadedElf {
    bytes: Vec<u8>,
}

impl LoadedElf {
    pub fn entry(&self) -> u64 {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(&self.bytes).expect("invalid ELF");

        elf.ehdr.e_entry
    }

    pub fn load_into(&self, bus: &mut Bus) -> Result<(), Trap> {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(&self.bytes).expect("invalid ELF");

        let segments = elf.segments().expect("failed to read program headers");

        for ph in segments.iter() {
            if ph.p_type != PT_LOAD {
                continue;
            }

            let file_start = ph.p_offset as usize;
            let file_end = file_start + ph.p_filesz as usize;

            let bytes = &self.bytes[file_start..file_end];

            // Copy initialized data
            bus.load(ph.p_paddr, bytes)?;

            // Zero-fill BSS
            if ph.p_memsz > ph.p_filesz {
                let start = ph.p_paddr + ph.p_filesz;

                for i in 0..(ph.p_memsz - ph.p_filesz) {
                    bus.write8(start + i, 0)?;
                }
            }
        }

        Ok(())
    }
}

pub fn load_execution_elf(path: impl AsRef<Path>) -> LoadedElf {
    LoadedElf { bytes: fs::read(path).expect("failed to read ELF") }
}
