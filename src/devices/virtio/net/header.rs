use crate::{cpu::memory::Memory, trap::Trap};

pub const VIRTIO_NET_HDR_F_NEEDS_CSUM: u8 = 1;
pub const VIRTIO_NET_HDR_F_DATA_VALID: u8 = 2;

pub const VIRTIO_NET_HDR_GSO_NONE: u8 = 0;
pub const VIRTIO_NET_HDR_GSO_TCPV4: u8 = 1;
pub const VIRTIO_NET_HDR_GSO_TCPV6: u8 = 4;

pub const VIRTIO_NET_HDR_SIZE: usize = 12;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VirtioNetHdr {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
}

impl VirtioNetHdr {
    pub fn read(mem: &Memory, addr: u64) -> Result<Self, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;
        let off = addr - RAM_BASE;
        let mut buf = [0u8; 12];
        mem.read_bulk(off, &mut buf)?;
        Ok(Self {
            flags: buf[0],
            gso_type: buf[1],
            hdr_len: u16::from_le_bytes([buf[2], buf[3]]),
            gso_size: u16::from_le_bytes([buf[4], buf[5]]),
            csum_start: u16::from_le_bytes([buf[6], buf[7]]),
            csum_offset: u16::from_le_bytes([buf[8], buf[9]]),
        })
    }

    pub fn write(&self, mem: &mut Memory, addr: u64) -> Result<(), Trap> {
        const RAM_BASE: u64 = 0x8000_0000;
        let off = addr - RAM_BASE;
        let buf: [u8; 12] = [
            self.flags,
            self.gso_type,
            self.hdr_len as u8,
            (self.hdr_len >> 8) as u8,
            self.gso_size as u8,
            (self.gso_size >> 8) as u8,
            self.csum_start as u8,
            (self.csum_start >> 8) as u8,
            self.csum_offset as u8,
            (self.csum_offset >> 8) as u8,
            0, 0,
        ];
        mem.load(off, &buf)
    }
}
