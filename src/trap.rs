#[derive(Debug)]
pub enum Trap {
    InstructionAccessFault,

    IllegalInstruction(u32),

    LoadAccessFault,
    LoadAddressMisaligned(u64),
    StoreAddressMisaligned(u64),
    InstructionAddressMisaligned(u64),

    StoreAccessFault,

    InstructionPageFault(u64),
    LoadPageFault(u64),
    StorePageFault(u64),

    Breakpoint,
    EcallFromUMode,
    EcallFromSMode,
    EcallFromMMode,
}
