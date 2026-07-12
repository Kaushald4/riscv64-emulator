# RISCV Emulator in rust 🦀 - Glasshart-emulator

[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-work%20in%20progress-yellow)](#roadmap)
[![RISC-V](https://img.shields.io/badge/ISA-RV64IMACFD-6c5ce7)](https://riscv.org/)

A 64-bit RISC-V emulator, written from scratch in Rust targeting RV64IMACFD, with the eventual goal of booting a real Linux distribution both natively and in the browser via WebAssembly.

> **Status:** I'm building this incrementally, and documenting the process as I go. Progress is tracked in the [Roadmap](#roadmap) below.

---

## Motivation behind taking this challenge

When I first came across [webvm.io](https://webvm.io/), I was fascinated how they have managed to get Linux running entirely in a web browser. At that time I had no idea how that was even possible, or how all the pieces fit together to make it work.

It's a great great project, but it only supports x86, and most modern packages have stopped shipping x86 binaries. I wanted to build something that could run a real 64-bit architecture one capable of running modern packages and languages partly to fill that gap, but mostly to satisfy my own curiosity about how all these components actually work together, and to learn something new in the process.

That's how I landed on RISC-V an open-source instruction set architecture, without decades of legacy baggage weighing it down the way x86 has.

It could have been enough for me to go through the specification from start to finish and be done with it. No, that wasn’t the point I wanted to experience the pain points, those things that you can only discover when writing a decoder and watching it do its thing wrong for hours

That's really the point of this project. Not just "an emulator that runs Linux," but a codebase where:

- The code stays readable enough that someone else learning RISC-V could open it up and follow along
- The end state is something genuinely fun to show off a real OS, booting inside a browser tab, running on an emulator I wrote myself

If you're also learning RISC-V or emulator internals, I'd genuinely love feedback or questions.

## Features

- [ ] RV64I - base integer instruction set
- [ ] RV64M - multiply / divide
- [ ] RV64C - compressed instructions
- [ ] RV64A - atomics
- [ ] RV64F/D - hardware floating point (single and double precision)
- [ ] Privilege levels (M/S/U) and trap handling
- [ ] CLINT - timer and software interrupts
- [ ] PLIC - platform-level interrupt controller
- [ ] UART (NS16550A) - serial console
- [ ] SBI - Supervisor Binary Interface, OpenSBI boot support
- [ ] Sv39 virtual memory - 3-level page table walker, TLB
- [ ] VirtIO block device
- [ ] Boots a minimal Linux (Alpine) rootfs
- [ ] WebAssembly build target - Linux, in a browser tab

## Roadmap

I'm building this in deliberate, testable phases rather than jumping straight for "boot Linux and hope." Each phase has to be provably correct before I let myself move to the next one that's the only way I've found to keep bugs from stacking on top of each other.

| Phase | Milestone                                           | Status      |
| :---- | :-------------------------------------------------- | :---------- |
| 0     | Fetch–decode–execute skeleton                       | Not started |
| 1     | Pass `riscv-tests` (`rv64ui` / `um` / `uc` / `ua`)  | Not started |
| 2     | Privilege levels, trap/exception handling, CLINT    | Not started |
| 3     | UART console, SBI implementation, OpenSBI boot      | Not started |
| 4     | Sv39 MMU — page table walker, TLB                   | Not started |
| 5     | RV64F/D floating point (pass `rv64uf` / `ud` tests) | Not started |
| 6     | PLIC, VirtIO block device, Alpine rootfs boot       | Not started |
| 7     | WASM build, browser boot                            | Not started |

I'll keep this table updated as phases land - check the commit history or releases for the details behind each one.

## Architecture

I will update as i progress.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- A RISC-V cross-compiler toolchain, if you want to build your own test binaries (`riscv64-unknown-elf-gcc` or similar) [riscv-gnu-toolchain](https://github.com/riscv-collab/riscv-gnu-toolchain)

### Build

```bash
git clone https://github.com/kaushald4/glasshart-emulator.git
cd glasshart-emulator
cargo build --release
```

### Test

```bash
cargo test
```

## References

I'm building this directly against the primary sources rather than secondhand explanations — it's slower going, but it's the only way to actually trust the result:

- [RISC-V Unprivileged ISA Specification](https://docs.riscv.org/reference/isa/v20260120/unpriv/unpriv-index.html)
- [RISC-V Privileged Architecture Specification](https://docs.riscv.org/reference/isa/v20260120/priv/priv-index.html)
- [RISC-V SBI Specification](https://docs.riscv.org/reference/sbi/_attachments/riscv-sbi.pdf)
- [riscv-tests](https://github.com/riscv-software-src/riscv-tests)
- [OpenSBI](https://github.com/riscv-software-src/opensbi)

## A note on this repo

This is a personal learning project, but I'm holding it to the same bar I'd want from production code tested, documented, and honest about what does and doesn't work yet. If you spot something wrong or have a suggestion, I'm genuinely happy to hear it open an issue or reach out.

## Additional References

- [Tinyemu](https://github.com/dearchap/tinyemu)
- [QEMU](https://github.com/qemu/QEMU)
- [webvm](https://webvm.io/)

## License

[MIT](LICENSE)
