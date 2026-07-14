use glasshart_emulator::decode;

fn main() {
    let inst = 0x00208733;
    let decoded = decode::decode(inst);
    println!("decoded inst: {:?}", decoded);
}
