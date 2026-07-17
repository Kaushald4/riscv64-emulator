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
