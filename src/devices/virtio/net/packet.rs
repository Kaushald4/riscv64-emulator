pub struct EthernetFrame<'a> {
    pub data: &'a [u8],
}

impl<'a> EthernetFrame<'a> {
    pub fn destination(&self) -> &[u8] {
        &self.data[..6]
    }

    pub fn source(&self) -> &[u8] {
        &self.data[6..12]
    }

    pub fn ethertype(&self) -> u16 {
        u16::from_be_bytes([self.data[12], self.data[13]])
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[14..]
    }
}
