#[derive(Debug)]
pub enum Trap {
    InstructionAccessFault,

    IllegalInstruction(u32),

    LoadAccessFault,
    LoadAddressMisaligned(u64),
    StoreAddressMisaligned(u64),
    InstructionAddressMisaligned(u64),

    StoreAccessFault,

    Breakpoint,
    EcallFromUMode,
    EcallFromSMode,
    EcallFromMMode,
}
