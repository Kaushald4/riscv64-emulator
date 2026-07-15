use softfloat_wrapper::{ExceptionFlags, F32, F64, Float};

use crate::cpu::{
    Cpu, ExecFlow, ExecResult,
    execute::fp::{canonicalize_f32_nan, clear_fflags, get_rounding_mode, rv_fcvt_i32, rv_fcvt_i64, rv_fcvt_u32, rv_fcvt_u64, update_fflags},
    f_register::FReg,
    register::Reg,
};

// memory related
pub fn flw(cpu: &mut Cpu, rd: FReg, rs1: Reg, imm: i64) -> ExecResult {
    let addr = cpu.regs.read(rs1).wrapping_add_signed(imm);

    let bits = cpu.bus.read32(addr)?;

    cpu.f_regs.write_f32_bits(rd, bits);

    Ok(ExecFlow::Next)
}
pub fn fsw(cpu: &mut Cpu, rs1: Reg, rs2: FReg, imm: i64) -> ExecResult {
    let addr = cpu.regs.read(rs1).wrapping_add_signed(imm);

    let bits = cpu.f_regs.read_f32_bits(rs2);

    cpu.bus.write32(addr, bits)?;

    Ok(ExecFlow::Next)
}

// moves
pub fn fmv_w_x(cpu: &mut Cpu, rd: FReg, rs1: Reg) -> ExecResult {
    let bits = cpu.regs.read(rs1) as u32;

    cpu.f_regs.write_f32_bits(rd, bits);

    Ok(ExecFlow::Next)
}
pub fn fmv_x_w(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let bits = cpu.f_regs.read_f32_raw_bits(rs1);
    cpu.regs.write(rd, bits as i32 as i64 as u64);

    Ok(ExecFlow::Next)
}

// sign injection
pub fn fsgnj_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f32_raw_bits(rs1);
    let b = cpu.f_regs.read_f32_raw_bits(rs2);

    let result = (a & 0x7fff_ffff) | (b & 0x8000_0000);

    cpu.f_regs.write_f32_bits(rd, result);

    Ok(ExecFlow::Next)
}
pub fn fsgnjn_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f32_raw_bits(rs1);
    let b = cpu.f_regs.read_f32_raw_bits(rs2);

    let result = (a & 0x7fff_ffff) | ((!b) & 0x8000_0000);

    cpu.f_regs.write_f32_bits(rd, result);

    Ok(ExecFlow::Next)
}
pub fn fsgnjx_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    let a = cpu.f_regs.read_f32_raw_bits(rs1);
    let b = cpu.f_regs.read_f32_raw_bits(rs2);

    let sign = (a ^ b) & 0x8000_0000;
    let result = (a & 0x7fff_ffff) | sign;

    cpu.f_regs.write_f32_bits(rd, result);

    Ok(ExecFlow::Next)
}

pub fn fclass_s(cpu: &mut Cpu, rd: Reg, rs1: FReg) -> ExecResult {
    let bits = cpu.f_regs.read_f32_bits(rs1);

    let sign = (bits >> 31) != 0;
    let exp = (bits >> 23) & 0xff;
    let frac = bits & 0x7fffff;

    let result = if exp == 0xff {
        if frac == 0 {
            if sign { 1 << 0 } else { 1 << 7 }
        } else {
            let quiet = (frac >> 22) & 1 != 0;
            if quiet { 1 << 9 } else { 1 << 8 }
        }
    } else if exp == 0 {
        if frac == 0 { if sign { 1 << 3 } else { 1 << 4 } } else { if sign { 1 << 2 } else { 1 << 5 } }
    } else {
        if sign { 1 << 1 } else { 1 << 6 }
    };

    cpu.regs.write(rd, result);

    Ok(ExecFlow::Next)
}

// softfloat arithmetic
pub fn fadd_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let result = a.add(b, rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fsub_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let result = canonicalize_f32_nan(a.sub(b, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmul_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let result = canonicalize_f32_nan(a.mul(b, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fdiv_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let result = canonicalize_f32_nan(a.div(b, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fsqrt_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));

    // let result = a.sqrt(rm);
    let result = canonicalize_f32_nan(a.sqrt(rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmin_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    if a.is_signaling_nan() || b.is_signaling_nan() {
        let mut flags = ExceptionFlags::default();
        flags.get();
        flags = ExceptionFlags::from_bits(flags.to_bits() | 0x10);
        flags.set();
    }

    let result = if a.is_nan() && b.is_nan() {
        F32::from_bits(0x7fc0_0000)
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
        if (a.to_bits() & 0x8000_0000) != 0 { a } else { b }
    };

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmax_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    if a.is_signaling_nan() || b.is_signaling_nan() {
        let mut flags = ExceptionFlags::default();
        flags.get();
        flags = ExceptionFlags::from_bits(flags.to_bits() | 0x10);
        flags.set();
    }

    let result = if a.is_nan() && b.is_nan() {
        F32::from_bits(0x7fc0_0000)
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
        if (a.to_bits() & 0x8000_0000) == 0 { a } else { b }
    };

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn feq_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    cpu.regs.write(rd, a.eq(&b) as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn flt_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    cpu.regs.write(rd, a.lt(&b) as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fle_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rs2: FReg) -> ExecResult {
    clear_fflags();

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    cpu.regs.write(rd, a.le(&b) as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_w_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f32_bits(rs1);
    let src = F32::from_bits(bits);

    // NaN must clamp to the positive max regardless of its sign bit.
    let negative = !src.is_nan() && (bits & 0x8000_0000) != 0;

    let value = rv_fcvt_i32(src.to_i32(rm, true), negative);

    cpu.regs.write(rd, value as i64 as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_wu_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f32_bits(rs1);
    let src = F32::from_bits(bits);

    // NaN must clamp to the positive max regardless of its sign bit.
    let negative = !src.is_nan() && (bits & 0x8000_0000) != 0;

    let value = rv_fcvt_u32(src.to_u32(rm, true), negative);

    cpu.regs.write(rd, (value as i32 as i64) as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_l_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f32_bits(rs1);
    let src = F32::from_bits(bits);

    // NaN must clamp to the positive max regardless of its sign bit.
    let negative = !src.is_nan() && (bits & 0x8000_0000) != 0;

    let value = rv_fcvt_i64(src.to_i64(rm, true), negative);

    cpu.regs.write(rd, value as u64);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_lu_s(cpu: &mut Cpu, rd: Reg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let bits = cpu.f_regs.read_f32_bits(rs1);
    let src = F32::from_bits(bits);

    // NaN must clamp to the positive max regardless of its sign bit.
    let negative = !src.is_nan() && (bits & 0x8000_0000) != 0;

    let value = rv_fcvt_u64(src.to_u64(rm, true), negative);

    cpu.regs.write(rd, value);

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
pub fn fcvt_s_w(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;
    let result = F32::from_i32(cpu.regs.read(rs1) as i32, rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_s_wu(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;
    let result = F32::from_u32(cpu.regs.read(rs1) as u32, rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_s_l(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;
    let result = F32::from_i64(cpu.regs.read(rs1) as i64, rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_s_lu(cpu: &mut Cpu, rd: FReg, rs1: Reg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;
    let result = F32::from_u64(cpu.regs.read(rs1), rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmadd_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));
    let c = F32::from_bits(cpu.f_regs.read_f32_bits(rs3));

    let result = canonicalize_f32_nan(a.fused_mul_add(b, c, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fmsub_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let mut c = F32::from_bits(cpu.f_regs.read_f32_bits(rs3));
    c = F32::from_bits(c.to_bits() ^ 0x8000_0000);

    let result = canonicalize_f32_nan(a.fused_mul_add(b, c, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fnmsub_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let mut a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));
    let c = F32::from_bits(cpu.f_regs.read_f32_bits(rs3));

    a = F32::from_bits(a.to_bits() ^ 0x8000_0000);

    let result = canonicalize_f32_nan(a.fused_mul_add(b, c, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fnmadd_s(cpu: &mut Cpu, rd: FReg, rs1: FReg, rs2: FReg, rs3: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let mut a = F32::from_bits(cpu.f_regs.read_f32_bits(rs1));
    let b = F32::from_bits(cpu.f_regs.read_f32_bits(rs2));

    let mut c = F32::from_bits(cpu.f_regs.read_f32_bits(rs3));

    a = F32::from_bits(a.to_bits() ^ 0x8000_0000);
    c = F32::from_bits(c.to_bits() ^ 0x8000_0000);

    let result = canonicalize_f32_nan(a.fused_mul_add(b, c, rm));

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}

pub fn fcvt_s_d(cpu: &mut Cpu, rd: FReg, rs1: FReg, rm: u8) -> ExecResult {
    clear_fflags();

    let rm = get_rounding_mode(cpu, rm)?;

    let a = F64::from_bits(cpu.f_regs.read_f64_bits(rs1));

    let result = a.to_f32(rm);

    cpu.f_regs.write_f32_bits(rd, result.to_bits());

    update_fflags(cpu);

    Ok(ExecFlow::Next)
}
