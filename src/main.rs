mod decode;
mod formats;
mod instruction;
mod opcode;
mod register;

fn main() {
    let inst = 0x01f0d713;
    let decoded = decode::decode(inst);
    println!("decoded inst: {:?}", decoded);
}
