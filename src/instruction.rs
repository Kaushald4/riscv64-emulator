/// RISC-V defines 32 general purpose integer registers (x0-x31).
/// Register x0 is hardwired to the constant value 0. Writes to x0 are
/// ignored, and reads from x0 always return 0.
enum Register {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
}

/// according to spec we have 6 base RISC-V instruction formats used by RV32I and RV64I.
/// - R: Register-register
/// - I: Immediate
/// - S: Store
/// - B: Branch
/// - U: Upper immediate
/// - J: Jump
#[derive(Clone, Copy)]
enum InstructionType {
    R,
    I,
    S,
    B,
    U,
    J,
}

enum Instruction {
    Addi {},
    Undefined,
}

const fn build_table() -> [Option<InstructionType>; 128] {
    let mut table = [None; 128];
    table[0b0010011] = Some(InstructionType::I);

    table
}

const ENCODING_TABLE: [Option<InstructionType>; 128] = build_table();
