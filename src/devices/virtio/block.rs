use std::fs;

use super::features::HOST_FEATURES;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtIOBlkConfig {
    pub capacity: u64,
    pub size_max: u32,
    pub seg_max: u32,
    pub geometry: [u8; 4],
    pub blk_size: u32,
}

pub struct VirtIOBlock {
    pub host_features: u64,
    pub config: VirtIOBlkConfig,

    pub image: Vec<u8>,
}

impl VirtIOBlock {
    pub fn new(path: &str) -> Self {
        let image = fs::read(path).expect("failed to load disk image");

        let capacity = image.len() as u64 / 512;

        Self {
            host_features: HOST_FEATURES,

            config: VirtIOBlkConfig { capacity, blk_size: 512, ..Default::default() },

            image,
        }
    }

    // pub fn read_sector(&self, sector: u64) -> &[u8] {
    //     let start = sector as usize * 512;
    //     let end = start + 512;

    //     &self.image[start..end]
    // }
    pub fn read_sector(&self, sector: u64, buffer: &mut [u8; 512]) {
        let start = sector as usize * 512;
        let end = start + 512;

        buffer.copy_from_slice(&self.image[start..end]);
    }
}
