use super::header::VirtIONetHeader;

pub struct NetworkPacket {
    pub header: VirtIONetHeader,
    pub data: Vec<u8>,
}
