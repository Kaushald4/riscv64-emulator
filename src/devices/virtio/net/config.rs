#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtIONetConfig {
    pub mac: [u8; 6],
    pub status: u16,
    pub max_virtqueue_pairs: u16,
    pub mtu: u16,
}
