#[derive(Default)]
pub struct VirtIONetConfig {
    pub mac: [u8; 6],
    pub status: u16,
    pub max_virtqueue_pairs: u16,
    pub mtu: u16,
}

impl VirtIONetConfig {
    pub fn read32(&self, offset: u64) -> u32 {
        match offset {
            0 => u32::from_le_bytes([self.mac[0], self.mac[1], self.mac[2], self.mac[3]]),
            4 => u32::from_le_bytes([self.mac[4], self.mac[5], self.status as u8, (self.status >> 8) as u8]),
            8 => self.max_virtqueue_pairs as u32,
            0x0c => self.mtu as u32,
            _ => 0,
        }
    }
}
