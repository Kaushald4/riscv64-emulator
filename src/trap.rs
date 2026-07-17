#[derive(Debug)]
pub enum Trap {
    InstructionAccessFault(u64),

    IllegalInstruction(u32),

    LoadAccessFault(u64),
    LoadAddressMisaligned(u64),
    StoreAddressMisaligned(u64),
    InstructionAddressMisaligned(u64),

    StoreAccessFault(u64),

    InstructionPageFault(u64),
    LoadPageFault(u64),
    StorePageFault(u64),

    Breakpoint,
    EcallFromUMode,
    EcallFromSMode,
    EcallFromMMode,
}
