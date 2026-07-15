use crate::cpu::execute::helper::addr;
use crate::cpu::register::Reg;
use crate::cpu::{Cpu, ExecFlow, ExecResult};
use crate::mmu::Mmu;
use crate::trap::Trap;

// ALU immediates
pub fn addi(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1) as i64;
    let result = a.wrapping_add(imm);
    cpu.regs.write(rd, result as u64);

    Ok(ExecFlow::Next)
}

pub fn slti(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1) as i64;
    cpu.regs.write(rd, (a < imm) as u64);

    Ok(ExecFlow::Next)
}

pub fn sltiu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, (a < imm as u64) as u64);

    Ok(ExecFlow::Next)
}

pub fn xori(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a ^ imm as u64);

    Ok(ExecFlow::Next)
}

pub fn ori(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a | imm as u64);

    Ok(ExecFlow::Next)
}

pub fn andi(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a & imm as u64);

    Ok(ExecFlow::Next)
}

pub fn slli(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a << shamt);

    Ok(ExecFlow::Next)
}

pub fn srli(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1);
    cpu.regs.write(rd, a >> shamt);

    Ok(ExecFlow::Next)
}

pub fn srai(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1) as i64;
    cpu.regs.write(rd, (a >> shamt) as u64);

    Ok(ExecFlow::Next)
}

// RV64-only immediate instructions (OP IMM 32)
pub fn addiw(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let a = cpu.regs.read(rs1) as i32;
    let result = a.wrapping_add(imm as i32);
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

pub fn slliw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1) as i32;
    let result = a.wrapping_shl(shamt);
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

pub fn srliw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1) as u32;
    let result = a >> shamt;
    cpu.regs.write(rd, (result as i32 as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn sraiw(cpu: &mut Cpu, rd: Reg, rs1: Reg, shamt: u32) -> ExecResult {
    let a = cpu.regs.read(rs1) as i32;
    let result = a >> shamt;
    cpu.regs.write(rd, result as i64 as u64);

    Ok(ExecFlow::Next)
}

// Alu register to register
pub fn add(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = cpu.regs.read(rs1).wrapping_add(cpu.regs.read(rs2));
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sub(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = cpu.regs.read(rs1).wrapping_sub(cpu.regs.read(rs2));
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sll(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let shamt = (cpu.regs.read(rs2) & 0x3f) as u32;
    let value = cpu.regs.read(rs1) << shamt;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn srl(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let shamt = (cpu.regs.read(rs2) & 0x3f) as u32;
    let value = cpu.regs.read(rs1) >> shamt;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sra(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let shamt = (cpu.regs.read(rs2) & 0x3f) as u32;
    let value = ((cpu.regs.read(rs1) as i64) >> shamt) as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn slt(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = ((cpu.regs.read(rs1) as i64) < (cpu.regs.read(rs2) as i64)) as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sltu(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = (cpu.regs.read(rs1) < cpu.regs.read(rs2)) as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn xor(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = cpu.regs.read(rs1) ^ cpu.regs.read(rs2);
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn or(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = cpu.regs.read(rs1) | cpu.regs.read(rs2);
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn and(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let value = cpu.regs.read(rs1) & cpu.regs.read(rs2);
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

// OP 32
pub fn addw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let lhs = cpu.regs.read(rs1) as u32;
    let rhs = cpu.regs.read(rs2) as u32;
    let value = lhs.wrapping_add(rhs) as i32 as i64 as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn subw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let lhs = cpu.regs.read(rs1) as u32;
    let rhs = cpu.regs.read(rs2) as u32;
    let value = lhs.wrapping_sub(rhs) as i32 as i64 as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sllw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let lhs = cpu.regs.read(rs1) as u32;
    let shamt = (cpu.regs.read(rs2) & 0x1f) as u32;
    let value = (lhs << shamt) as i32 as i64 as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn srlw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let lhs = cpu.regs.read(rs1) as u32;
    let shamt = (cpu.regs.read(rs2) & 0x1f) as u32;
    let value = (lhs >> shamt) as i32 as i64 as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn sraw(cpu: &mut Cpu, rd: Reg, rs1: Reg, rs2: Reg) -> ExecResult {
    let lhs = cpu.regs.read(rs1) as i32;
    let shamt = (cpu.regs.read(rs2) & 0x1f) as u32;
    let value = (lhs >> shamt) as i64 as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

// Loads
pub fn lb(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read8(cpu, addr)? as i8 as i64 as u64;

    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn lbu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read8(cpu, addr)? as u64;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

pub fn lh(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read16(cpu, addr)?;
    cpu.regs.write(rd, (value as i16 as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn lhu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read16(cpu, addr)?;
    cpu.regs.write(rd, value as u64);

    Ok(ExecFlow::Next)
}

pub fn lw(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read32(cpu, addr)?;

    cpu.regs.write(rd, (value as i32 as i64) as u64);

    Ok(ExecFlow::Next)
}

pub fn lwu(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read32(cpu, addr)?;
    cpu.regs.write(rd, value as u64);

    Ok(ExecFlow::Next)
}

pub fn ld(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let value = Mmu::read64(cpu, addr)?;
    cpu.regs.write(rd, value);

    Ok(ExecFlow::Next)
}

// Branches
pub fn beq(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if cpu.regs.read(rs1) == cpu.regs.read(rs2) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn bne(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if cpu.regs.read(rs1) != cpu.regs.read(rs2) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn blt(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if (cpu.regs.read(rs1) as i64) < (cpu.regs.read(rs2) as i64) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn bge(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if (cpu.regs.read(rs1) as i64) >= (cpu.regs.read(rs2) as i64) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn bltu(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if cpu.regs.read(rs1) < cpu.regs.read(rs2) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn bgeu(cpu: &mut Cpu, rs1: Reg, rs2: Reg, imm: i64) -> ExecResult {
    if cpu.regs.read(rs1) >= cpu.regs.read(rs2) { Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm))) } else { Ok(ExecFlow::Next) }
}

pub fn jal(cpu: &mut Cpu, rd: Reg, imm: i64) -> ExecResult {
    let return_addr = cpu.pc + cpu.current_instruction_length as u64;

    cpu.regs.write(rd, return_addr);

    Ok(ExecFlow::Jump(cpu.pc.wrapping_add_signed(imm)))
}

pub fn jalr(cpu: &mut Cpu, rd: Reg, rs1: Reg, imm: i64) -> ExecResult {
    let return_addr = cpu.pc + cpu.current_instruction_length as u64;

    let target = cpu.regs.read(rs1).wrapping_add_signed(imm) & !1;

    if target & 1 != 0 {
        return Err(Trap::InstructionAddressMisaligned(target));
    }

    cpu.regs.write(rd, return_addr);

    Ok(ExecFlow::Jump(target))
}
