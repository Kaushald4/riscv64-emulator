use crate::decoder::decode;

mod decoder;
mod instruction;

fn main() {
    let inst = 0x01900193;
    let decoded = decode(inst);
}
