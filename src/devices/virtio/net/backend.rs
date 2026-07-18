pub trait NetworkBackend {
    fn send(&mut self, packet: &[u8]) -> Result<(), ()>;
    fn recv(&mut self) -> Option<Vec<u8>>;
}
