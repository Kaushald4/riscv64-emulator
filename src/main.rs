use glasshart_emulator::cpu::{self, register::Reg};

use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();

    let instruction: u32 = 0x00500093;

    let program = [
        0x00500093, // addi x1, x0, 5
        0x00308113, // addi x2, x1, 3
    ];

    for instr in program {
        cpu.step(instr);
    }

    println!("PC: {}", cpu.pc);
    println!("x1: {}", cpu.regs.read(Reg::new(1)));
    println!("x2: {}", cpu.regs.read(Reg::new(2)));
}
