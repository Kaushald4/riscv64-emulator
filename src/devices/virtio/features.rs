pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
pub const VIRTIO_RING_F_INDIRECT_DESC: u64 = 1 << 28;
pub const VIRTIO_RING_F_EVENT_IDX: u64 = 1 << 29;

pub const VIRTIO_BLK_F_RO: u64 = 1 << 5;
pub const VIRTIO_BLK_F_FLUSH: u64 = 1 << 9;
pub const VIRTIO_BLK_F_CONFIG_WCE: u64 = 1 << 11;

// TODO: for later implement below
// VIRTIO_RING_F_INDIRECT_DESC
// VIRTIO_RING_F_EVENT_IDX
pub const HOST_FEATURES: u64 = VIRTIO_F_VERSION_1 | VIRTIO_BLK_F_FLUSH;
