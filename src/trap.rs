#[derive(Debug)]
pub enum Trap {
    InstructionAddressMisaligned,
    InstructionAccessFault,

    IllegalInstruction(u32),

    LoadAddressMisaligned,
    LoadAccessFault,

    StoreAddressMisaligned,
    StoreAccessFault,

    Breakpoint,
    EcallFromUMode,
    EcallFromSMode,
    EcallFromMMode,
}
