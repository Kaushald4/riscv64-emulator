#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtIOBlkConfig {
    pub capacity: u64,
    pub size_max: u32,
    pub seg_max: u32,
    pub geometry: [u8; 4],
    pub blk_size: u32,
}
