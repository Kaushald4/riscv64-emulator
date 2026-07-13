mod decode;
mod formats;
mod instruction;
mod opcode;
mod register;

fn main() {
    let inst = 0x00208733;
    let decoded = decode::decode(inst);
    println!("decoded inst: {:?}", decoded);
}
