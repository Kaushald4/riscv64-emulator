use crate::instruction::Instruction;

pub fn decode_compressed(raw: u16) -> Instruction {
    Instruction::Undefined { raw: raw as u32 }
}
