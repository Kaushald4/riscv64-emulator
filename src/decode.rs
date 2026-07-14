mod c_formats;
mod compressed;
mod formats;
mod rv32_64;

use crate::{
    decode::{compressed::decode_compressed, rv32_64::decode_normal},
    instruction::DecodedInstruction,
};

pub fn decode(raw: u32) -> DecodedInstruction {
    let first_half = raw as u16;

    if first_half & 0b11 != 0b11 {
        DecodedInstruction { instruction: decode_compressed(first_half), length: 2 }
    } else {
        DecodedInstruction { instruction: decode_normal(raw), length: 4 }
    }
}
