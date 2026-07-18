use crate::{cpu::memory::Memory, trap::Trap};

pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;
pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestType {
    In = 0,
    Out = 1,
    Flush = 4,
    GetId = 8,
    Unknown = u32::MAX,
}

impl From<u32> for RequestType {
    fn from(value: u32) -> Self {
        match value {
            0 => RequestType::In,
            1 => RequestType::Out,
            4 => RequestType::Flush,
            8 => RequestType::GetId,
            _ => RequestType::Unknown,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct VirtIOBlkReqHeader {
    pub request_type: u32,
    pub reserved: u32,
    pub sector: u64,
}

impl VirtIOBlkReqHeader {
    pub fn read(mem: &Memory, addr: u64) -> Result<Self, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;
        let off = addr - RAM_BASE;
        Ok(Self {
            request_type: mem.read32(off)?,
            reserved: mem.read32(off + 4)?,
            sector: mem.read64(off + 8)?,
        })
    }
}
