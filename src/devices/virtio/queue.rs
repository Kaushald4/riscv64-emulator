use crate::{cpu::memory::Memory, trap::Trap};

use super::descriptor::VirtqDesc;

#[derive(Clone, Debug)]
pub struct VirtQueue {
    pub size: u16,
    pub ready: bool,

    pub desc_table: u64,
    pub avail_ring: u64,
    pub used_ring: u64,

    pub last_avail_idx: u16,

    /// whether `VIRTIO_RING_F_EVENT_IDX` was negotiated for this device.
    /// Set by the MMIO layer once `DRIVER_FEATURES` is written by the
    /// guest. When true, the transport layer reads `used_event` from the
    /// avail ring to decide whether to fire an interrupt (per virtio 1.1
    /// 2.7). when false, we always interrupt (subject to the
    /// NO_INTERRUPT flag).
    pub event_idx_enabled: bool,
}

impl Default for VirtQueue {
    fn default() -> Self {
        Self {
            size: 128,
            ready: false,
            desc_table: 0,
            avail_ring: 0,
            used_ring: 0,
            last_avail_idx: 0,
            event_idx_enabled: false,
        }
    }
}

impl VirtQueue {
    pub fn pop_descriptor(&mut self, mem: &Memory) -> Result<Option<(u16, VirtqDesc)>, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;

        if !self.ready {
            return Ok(None);
        }

        let avail_idx = mem.read16(self.avail_ring - RAM_BASE + 2)?;

        if avail_idx == self.last_avail_idx {
            return Ok(None);
        }

        let ring_addr = self.avail_ring + 4 + ((self.last_avail_idx as u64 % self.size as u64) * 2);
        let desc_head = mem.read16(ring_addr - RAM_BASE)?;

        self.last_avail_idx = self.last_avail_idx.wrapping_add(1);

        let desc_addr = self.desc_table + (desc_head as u64 * 16);
        let desc = VirtqDesc::read(mem, desc_addr)?;
        Ok(Some((desc_head, desc)))
    }

    /// check whether the driver has posted any buffers we haven't consumed
    /// yet — WITHOUT advancing the avail pointer. Used by the net RX path
    /// to decide whether to poll the backend: if there's nowhere to put a
    /// received frame, we must NOT call `receive()` (it destructively
    /// dequeues from the WebRTC queue, so a frame with no destination
    /// buffer is lost forever).
    pub fn has_available(&self, mem: &Memory) -> Result<bool, Trap> {
        const RAM_BASE: u64 = 0x8000_0000;
        if !self.ready {
            return Ok(false);
        }
        let avail_idx = mem.read16(self.avail_ring - RAM_BASE + 2)?;
        Ok(avail_idx != self.last_avail_idx)
    }

    /// undo a `pop_descriptor` by decrementing the avail index. Safe to
    /// call once after a pop, as long as no other pop has happened in
    /// between. Used by the net RX path to "put back" a buffer when the
    /// backend had no frame to fill it with — without this, the buffer
    /// would be permanently lost to the guest.
    pub fn unpop(&mut self) {
        self.last_avail_idx = self.last_avail_idx.wrapping_sub(1);
    }
}
