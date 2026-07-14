use glasshart_emulator::{
    cpu::{self, execute::rv64i, register::Reg},
    trap::Trap,
};

use cpu::Cpu;

fn main() -> Result<(), Trap> {
    let mut cpu = Cpu::new();

    cpu.bus.write32(0x8000_0000, 0x12345678)?;

    cpu.regs.write(Reg::new(1), 0x8000_0000);

    rv64i::lw(&mut cpu, Reg::new(2), Reg::new(1), 0)?;

    assert_eq!(cpu.regs.read(Reg::new(2)), 0x0000000012345678,);

    println!("LW passed");

    Ok(())
}
