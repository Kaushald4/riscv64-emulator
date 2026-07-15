use softfloat_wrapper::{ExceptionFlags, F32, F64, Float};

use crate::cpu::{
    Cpu, ExecFlow, ExecResult,
    execute::{
        fp::{clear_fflags, get_rounding_mode, rv_fcvt_i32, rv_fcvt_i64, rv_fcvt_u32, rv_fcvt_u64, update_fflags},
        helper::addr,
    },
    f_register::FReg,
    register::Reg,
};
// memory
pub fn fld(cpu: &mut Cpu, rd: FReg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let bits = cpu.bus.read64(addr)?;

    cpu.f_regs.write_f64_bits(rd, bits);

    Ok(ExecFlow::Next)
}

pub fn fsd(cpu: &mut Cpu, rs1: Reg, rs2: FReg, imm: i64) -> ExecResult {
    let addr = addr(cpu, rs1, imm);
    let bits = cpu.f_regs.read_f64_bits(rs2);

    cpu.bus.write64(addr, bits)?;

    Ok(ExecFlow::Next)
}
// moves
pub fn fmv_d_x(cpu: &mut Cpu, rd: FReg, rs1: Reg) -> ExecResult {
    let bits = cpu.regs.read(rs1);

    cpu.f_regs.write_f64_bits(rd, bits);

    Ok(ExecFlow::Next)
}

pub fn fmv_x_d(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let bits = cpu.f_regs.read_f64_bits(rs1);

    cpu.regs.write(rd, bits);

    Ok(ExecFlow::Next)
}

// arithmetic
pub fn fadd_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let mut result = a.add(b, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fsub_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let mut result = a.sub(b, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmul_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let mut result = a.mul(b, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fdiv_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let mut result = a.div(b, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fsqrt_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));

    let mut result = a.sqrt(rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fsgnj_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f64_bits(rs1);
    let b = cpu.f_regs.read_f64_bits(rs2);

    let result = (a & 0x7fff_ffff_ffff_ffff) | (b & 0x8000_0000_0000_0000);

    cpu.f_regs.write_f64_bits(rd, result);

    Ok(ExecFlow::Next)
}

pub fn fsgnjn_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f64_bits(rs1);
    let b = cpu.f_regs.read_f64_bits(rs2);

    let result = (a & 0x7fff_ffff_ffff_ffff) | ((!b) & 0x8000_0000_0000_0000);

    cpu.f_regs.write_f64_bits(rd, result);

    Ok(ExecFlow::Next)
}

pub fn fsgnjx_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f64_bits(rs1);
    let b = cpu.f_regs.read_f64_bits(rs2);

    let sign = (a ^ b) & 0x8000_0000_0000_0000;
    let result = (a & 0x7fff_ffff_ffff_ffff) | sign;

    cpu.f_regs.write_f64_bits(rd, result);

    Ok(ExecFlow::Next)
}
pub fn fmin_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    if a.is_signaling_nan() || b.is_signaling_nan() {
        let mut flags = ExceptionFlags::default();
        flags.get();
        flags = ExceptionFlags::from_bits(flags.to_bits() | 0x10);
        flags.set();
    }

    let result = if a.is_nan() && b.is_nan() {
        F64::from_bits(0x7ff8_0000_0000_0000)
    } else if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a.lt(&b) {
        a
    } else if b.lt(&a) {
        b
    } else {
        // equal: prefer -0
        if (a.to_bits() & 0x8000_0000_0000_0000) != 0 { a } else { b }
    };

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmax_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    if a.is_signaling_nan() || b.is_signaling_nan() {
        let mut flags = ExceptionFlags::default();
        flags.get();
        flags = ExceptionFlags::from_bits(flags.to_bits() | 0x10);
        flags.set();
    }

    let result = if a.is_nan() && b.is_nan() {
        F64::from_bits(0x7ff8_0000_0000_0000)
    } else if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a.lt(&b) {
        b
    } else if b.lt(&a) {
        a
    } else {
        // equal: prefer +0
        if (a.to_bits() & 0x8000_0000_0000_0000) == 0 { a } else { b }
    };

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn feq_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let result = if a.eq(&b) { 1 } else { 0 };

    cpu.regs.write(rd, result);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn flt_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let result = if a.lt(&b) { 1 } else { 0 };

    cpu.regs.write(rd, result);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fle_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));

    let result = if a.le(&b) { 1 } else { 0 };

    cpu.regs.write(rd, result);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fclass_d(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let bits = a.to_bits();

    let sign = (bits >> 63) != 0;
    let exp = (bits >> 52) & 0x7ff;
    let frac = bits & 0x000f_ffff_ffff_ffff;

    let result = if exp == 0x7ff {
        if frac == 0 { if sign { 1 << 0 } else { 1 << 7 } } else { if a.is_signaling_nan() { 1 << 8 } else { 1 << 9 } }
    } else if exp == 0 {
        if frac == 0 { if sign { 1 << 3 } else { 1 << 4 } } else { if sign { 1 << 2 } else { 1 << 5 } }
    } else {
        if sign { 1 << 1 } else { 1 << 6 }
    };

    cpu.regs.write(rd, result);

    Ok(ExecFlow::Next)
}
pub fn fcvt_w_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f64_bits(rs1);
    let src = F64::from_bits(bits);

    let value = rv_fcvt_i32(src.to_i32(rm, true), src.is_nan(), (bits & 0x8000_0000_0000_0000) != 0);

    cpu.regs.write(rd, value as i64 as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_wu_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f64_bits(rs1);
    let src = F64::from_bits(bits);

    let value = rv_fcvt_u32(src.to_u32(rm, true), src.is_nan(), (bits & 0x8000_0000_0000_0000) != 0);

    cpu.regs.write(rd, (value as i32 as i64) as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_l_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f64_bits(rs1);
    let src = F64::from_bits(bits);

    let value = rv_fcvt_i64(src.to_i64(rm, true), src.is_nan(), (bits & 0x8000_0000_0000_0000) != 0);

    cpu.regs.write(rd, value as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_lu_d(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f64_bits(rs1);
    let src = F64::from_bits(bits);

    let value = rv_fcvt_u64(src.to_u64(rm, true), src.is_nan(), (bits & 0x8000_0000_0000_0000) != 0);

    cpu.regs.write(rd, value);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_d_w(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let value = cpu.regs.read(rs1) as i32;

    let mut result = F64::from_i32(value, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_d_wu(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let value = cpu.regs.read(rs1) as u32;

    let mut result = F64::from_u32(value, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_d_l(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let value = cpu.regs.read(rs1) as i64;

    let mut result = F64::from_i64(value, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_d_lu(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let value = cpu.regs.read(rs1);

    let mut result = F64::from_u64(value, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_s_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let src = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));

    let mut result = src.to_f32(rm);

    if result.is_nan() {
        result = F32::from_bits(0x7fc0_0000);
    }

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_d_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let src = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));

    let mut result = src.to_f64(rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmadd_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));
    let c = F64::from_bits(cpu.f_regs.read_f64_bits(rs3));

    let mut result = a.fused_mul_add(b, c, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmsub_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));
    let c = F64::from_bits(cpu.f_regs.read_f64_bits(rs3));

    let neg_c = F64::from_bits(c.to_bits() ^ 0x8000_0000_0000_0000);

    let mut result = a.fused_mul_add(b, neg_c, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fnmsub_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));
    let c = F64::from_bits(cpu.f_regs.read_f64_bits(rs3));

    let neg_a = F64::from_bits(a.to_bits() ^ 0x8000_0000_0000_0000);

    let mut result = neg_a.fused_mul_add(b, c, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fnmadd_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));
    let b = F64::from_bits(cpu.f_regs.read_f64_bits(rs2));
    let c = F64::from_bits(cpu.f_regs.read_f64_bits(rs3));

    let neg_a = F64::from_bits(a.to_bits() ^ 0x8000_0000_0000_0000);
    let neg_c = F64::from_bits(c.to_bits() ^ 0x8000_0000_0000_0000);

    let mut result = neg_a.fused_mul_add(b, neg_c, rm);

    if result.is_nan() {
        result = F64::from_bits(0x7ff8_0000_0000_0000);
    }

    cpu.f_regs.write_f64_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
