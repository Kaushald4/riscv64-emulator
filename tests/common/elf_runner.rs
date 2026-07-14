use std::{
    fs,
    path::{Path, PathBuf},
};

use elf::{ElfBytes, abi::PT_LOAD, endian::AnyEndian};

use glasshart_emulator::{cpu::bus::Bus, trap::Trap};

pub struct LoadedElf {
    bytes: Vec<u8>,
    tohost: Option<u64>,
}

impl LoadedElf {
    pub fn entry(&self) -> u64 {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(&self.bytes).expect("invalid ELF");

        elf.ehdr.e_entry
    }

    pub fn tohost(&self) -> Option<u64> {
        self.tohost
    }

    pub fn load_into(&self, bus: &mut Bus) -> Result<(), Trap> {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(&self.bytes).expect("invalid ELF");

        let segments = elf.segments().expect("failed to read program headers");

        for ph in segments.iter() {
            if ph.p_type != PT_LOAD {
                continue;
            }

            let start = ph.p_offset as usize;
            let end = start + ph.p_filesz as usize;

            bus.load(ph.p_paddr, &self.bytes[start..end])?;

            if ph.p_memsz > ph.p_filesz {
                let bss = ph.p_paddr + ph.p_filesz;

                for i in 0..(ph.p_memsz - ph.p_filesz) {
                    bus.write8(bss + i, 0)?;
                }
            }
        }

        Ok(())
    }
}

pub fn load_execution_elf(path: impl AsRef<Path>) -> LoadedElf {
    let path = path.as_ref();

    let bytes = fs::read(path).expect("failed to read ELF");

    let tohost = parse_tohost_from_dump(path);

    LoadedElf { bytes, tohost }
}

fn parse_tohost_from_dump(elf_path: &Path) -> Option<u64> {
    let mut dump = PathBuf::from(elf_path);
    dump.set_extension("dump");

    let text = fs::read_to_string(dump).ok()?;

    for line in text.lines() {
        if !line.contains("<tohost>") {
            continue;
        }

        let hash = line.find('#')?;
        let end = line.find("<tohost>")?;

        let addr = line[hash + 1..end].trim();

        return u64::from_str_radix(addr, 16).ok();
    }

    None
}
