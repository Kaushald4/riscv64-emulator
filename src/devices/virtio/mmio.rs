use super::{block::VirtIOBlock, config::*, queue::VirtQueue};

pub struct VirtIOMmio {
    pub device: VirtIOBlock,

    pub status: u32,

    pub device_features_sel: u32,
    pub driver_features_sel: u32,

    pub driver_features: u64,

    pub queue_sel: u32,

    pub interrupt_status: u32,

    pub queues: Vec<VirtQueue>,
    pub queue_notify: Option<u16>,
}

impl VirtIOMmio {
    pub fn new() -> Self {
        Self {
            device: VirtIOBlock::new("kernel/base.img"),

            status: 0,

            device_features_sel: 0,
            driver_features_sel: 0,

            driver_features: 0,

            queue_sel: 0,

            interrupt_status: 0,

            queues: vec![VirtQueue::default()],
            queue_notify: None,
        }
    }

    pub fn read32(&self, offset: u64) -> u32 {
        if offset >= CONFIG_SPACE {
            return self.read_config_u32(offset - CONFIG_SPACE);
        }
        match offset {
            MAGIC_VALUE => VIRTIO_MAGIC,
            VERSION => VIRTIO_VERSION,
            DEVICE_ID => VIRTIO_DEVICE_BLOCK,
            VENDOR_ID => VIRTIO_VENDOR_ID,

            DEVICE_FEATURES => {
                if self.device_features_sel == 0 {
                    self.device.host_features as u32
                } else {
                    (self.device.host_features >> 32) as u32
                }
            }

            QUEUE_NUM_MAX => 128,

            STATUS => self.status,

            INTERRUPT_STATUS => self.interrupt_status,

            CONFIG_GENERATION => 0,

            _ => 0,
        }
    }

    pub fn write32(&mut self, offset: u64, value: u32) {
        match offset {
            DEVICE_FEATURES_SEL => {
                self.device_features_sel = value;
            }

            DRIVER_FEATURES_SEL => {
                self.driver_features_sel = value;
            }

            DRIVER_FEATURES => {
                if self.driver_features_sel == 0 {
                    self.driver_features = (self.driver_features & !0xffff_ffff) | value as u64;
                } else {
                    self.driver_features = (self.driver_features & 0xffff_ffff) | ((value as u64) << 32);
                }
            }

            STATUS => {
                self.status = value;
            }

            QUEUE_SEL => {
                self.queue_sel = value;
            }

            QUEUE_NUM => {
                self.queues[self.queue_sel as usize].size = value as u16;
            }

            QUEUE_READY => {
                self.queues[self.queue_sel as usize].ready = value != 0;
            }

            QUEUE_DESC_LOW => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.desc_table = (q.desc_table & 0xffff_ffff_0000_0000) | value as u64;
            }

            QUEUE_DESC_HIGH => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.desc_table = (q.desc_table & 0x0000_0000_ffff_ffff) | ((value as u64) << 32);
            }

            QUEUE_DRIVER_LOW => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.avail_ring = (q.avail_ring & 0xffff_ffff_0000_0000) | value as u64;
            }

            QUEUE_DRIVER_HIGH => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.avail_ring = (q.avail_ring & 0x0000_0000_ffff_ffff) | ((value as u64) << 32);
            }

            QUEUE_DEVICE_LOW => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.used_ring = (q.used_ring & 0xffff_ffff_0000_0000) | value as u64;
            }

            QUEUE_DEVICE_HIGH => {
                let q = &mut self.queues[self.queue_sel as usize];
                q.used_ring = (q.used_ring & 0x0000_0000_ffff_ffff) | ((value as u64) << 32);
            }

            QUEUE_NOTIFY => {
                self.notify_queue(value as u16);
            }

            INTERRUPT_ACK => {
                self.interrupt_status &= !value;
            }

            _ => {}
        }
    }
}

impl VirtIOMmio {
    fn read_config_u32(&self, offset: u64) -> u32 {
        match offset {
            0x00 => self.device.config.capacity as u32,
            0x04 => (self.device.config.capacity >> 32) as u32,
            0x08 => self.device.config.size_max,
            0x0c => self.device.config.seg_max,
            0x14 => self.device.config.blk_size,
            _ => 0,
        }
    }
}

impl VirtIOMmio {
    pub fn notify_queue(&mut self, queue: u16) {
        self.queue_notify = Some(queue);
    }

    pub fn take_queue_notify(&mut self) -> Option<u16> {
        self.queue_notify.take()
    }
}
