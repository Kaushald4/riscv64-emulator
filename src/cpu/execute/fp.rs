use crate::{cpu::Cpu, trap::Trap};
use softfloat_wrapper::{ExceptionFlags, F32, Float, RoundingMode};

#[inline]
pub(crate) fn get_rounding_mode(cpu: &Cpu, rm: u8) -> Result<RoundingMode, Trap> {
    let rm = if rm == 0b111 { cpu.csr.frm as u8 } else { rm };

    match rm {
        0b000 => Ok(RoundingMode::TiesToEven),
        0b001 => Ok(RoundingMode::TowardZero),
        0b010 => Ok(RoundingMode::TowardNegative),
        0b011 => Ok(RoundingMode::TowardPositive),
        0b100 => Ok(RoundingMode::TiesToAway),
        _ => Err(Trap::IllegalInstruction(0)),
    }
}

#[inline]
pub(crate) fn clear_fflags() {
    ExceptionFlags::default().set();
}

#[inline]
pub(crate) fn update_fflags(cpu: &mut Cpu) {
    let mut flags = ExceptionFlags::default();
    flags.get();

    cpu.csr.fflags |= flags.to_bits() as u64;
    cpu.csr.fcsr = (cpu.csr.frm << 5) | cpu.csr.fflags;
}

#[inline]
pub(crate) fn canonicalize_f32_nan(result: F32) -> F32 {
    if result.is_nan() { F32::from_bits(0x7fc0_0000) } else { result }
}

#[inline]
pub(crate) fn rv_fcvt_i32(value: i32, negative: bool) -> i32 {
    let mut flags = ExceptionFlags::default();
    flags.get();

    if !flags.is_invalid() {
        return value;
    }

    if negative { i32::MIN } else { i32::MAX }
}

#[inline]
pub(crate) fn rv_fcvt_u32(value: u32, negative: bool) -> u32 {
    let mut flags = ExceptionFlags::default();
    flags.get();

    if !flags.is_invalid() {
        return value;
    }

    if negative { 0 } else { u32::MAX }
}

#[inline]
pub(crate) fn rv_fcvt_i64(value: i64, negative: bool) -> i64 {
    let mut flags = ExceptionFlags::default();
    flags.get();

    if !flags.is_invalid() {
        return value;
    }

    if negative { i64::MIN } else { i64::MAX }
}

#[inline]
pub(crate) fn rv_fcvt_u64(value: u64, negative: bool) -> u64 {
    let mut flags = ExceptionFlags::default();
    flags.get();

    if !flags.is_invalid() {
        return value;
    }

    if negative { 0 } else { u64::MAX }
}
