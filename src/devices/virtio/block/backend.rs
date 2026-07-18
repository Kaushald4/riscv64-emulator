pub trait BlockBackend {
    fn sector_count(&self) -> u64;
    fn read(&self, offset: usize, len: usize) -> Vec<u8>;
    fn write(&mut self, offset: usize, data: &[u8]);
}

/// File-backed storage for native builds.
#[cfg(not(target_arch = "wasm32"))]
pub struct FileBackend {
    data: Vec<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileBackend {
    pub fn new(path: &str) -> Self {
        let data = std::fs::read(path).expect("failed to load disk image");
        Self { data }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BlockBackend for FileBackend {
    fn sector_count(&self) -> u64 {
        self.data.len() as u64 / 512
    }

    fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        self.data[offset..offset + len].to_vec()
    }

    fn write(&mut self, offset: usize, data: &[u8]) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }
}

/// In-memory backend for tests / WASM.
pub struct MemoryBackend {
    data: Vec<u8>,
}

impl MemoryBackend {
    pub fn new(sectors: u64) -> Self {
        Self { data: vec![0u8; sectors as usize * 512] }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl BlockBackend for MemoryBackend {
    fn sector_count(&self) -> u64 {
        self.data.len() as u64 / 512
    }

    fn read(&self, offset: usize, len: usize) -> Vec<u8> {
        self.data[offset..offset + len].to_vec()
    }

    fn write(&mut self, offset: usize, data: &[u8]) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }
}
