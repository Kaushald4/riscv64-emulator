#[cfg(test)]
mod tests {
    use glasshart_emulator::mmu::satp::{Satp, SatpMode};

    use super::*;

    #[test]
    fn decode_bare() {
        let satp = Satp::new(0);

        assert_eq!(satp.mode(), SatpMode::Bare);
        assert_eq!(satp.asid(), 0);
        assert_eq!(satp.ppn(), 0);
    }

    #[test]
    fn decode_sv39() {
        let bits = (8u64 << 60) | (0x1234u64 << 44) | 0x123456789ab;

        let satp = Satp::new(bits);

        assert_eq!(satp.mode(), SatpMode::Sv39);
        assert_eq!(satp.asid(), 0x1234);
        assert_eq!(satp.ppn(), 0x123456789ab);
    }

    #[test]
    fn decode_reserved_mode() {
        let satp = Satp::new(15u64 << 60);

        assert_eq!(satp.mode(), SatpMode::Reserved(15));
    }
}
