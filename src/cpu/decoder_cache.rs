use crate::instruction::DecodedInstruction;

#[derive(Clone, Copy)]
pub struct DecodeEntry {
    pub pc: u64,
    pub valid: bool,
    pub decoded: DecodedInstruction,
    pub length: u8,
}

impl Default for DecodeEntry {
    fn default() -> Self {
        Self {
            pc: 0,
            valid: false,
            decoded: DecodedInstruction::default(),
            length: 0,
        }
    }
}

/// Maximum number of instructions in a basic block. Most real basic
/// blocks are < 20 instructions. 32 gives headroom for unrolled loops.
pub const MAX_BLOCK_SIZE: usize = 32;

/// A basic block: a sequence of pre-decoded instructions that execute
/// sequentially until a control flow instruction (branch, jump, CSR,
/// system). On a cache hit, the entire block is executed in a tight
/// loop without per-instruction fetch, decode, or cache lookup.
///
/// The block is invalidated via a generation counter — when any
/// translation-changing event occurs (TLB flush, privilege change,
/// satp write), `block_gen` is incremented, invalidating all blocks.
#[derive(Clone)]
pub struct BasicBlock {
    pub pc: u64,
    pub generation: u64,
    pub count: u8,
    pub decoded: [DecodedInstruction; MAX_BLOCK_SIZE],
    pub lengths: [u8; MAX_BLOCK_SIZE],
}

impl Default for BasicBlock {
    fn default() -> Self {
        Self {
            pc: 0,
            generation: 0,
            count: 0,
            decoded: [DecodedInstruction::default(); MAX_BLOCK_SIZE],
            lengths: [0; MAX_BLOCK_SIZE],
        }
    }
}
