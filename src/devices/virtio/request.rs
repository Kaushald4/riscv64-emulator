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
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct VirtIOBlkReqHeader {
    pub request_type: u32,
    pub reserved: u32,
    pub sector: u64,
}
