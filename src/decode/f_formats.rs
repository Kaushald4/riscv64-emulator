use crate::cpu::f_register::FReg;

#[inline]
pub const fn frd(raw: u32) -> FReg {
    FReg::new((raw >> 7) & 0x1f)
}

#[inline]
pub const fn frs2(raw: u32) -> FReg {
    FReg::new((raw >> 20) & 0x1f)
}

#[inline]
pub const fn frs1(raw: u32) -> FReg {
    FReg::new((raw >> 15) & 0x1f)
}

#[inline]
pub const fn frs3(raw: u32) -> FReg {
    FReg::new((raw >> 27) & 0x1f)
}
