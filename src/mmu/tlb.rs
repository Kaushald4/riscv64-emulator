#[derive(Clone, Copy, Debug, Default)]
pub struct TlbEntry {
    pub vpn: u64,
    pub asid: u16,
    pub is_global: bool,
    pub valid: bool,
    pub pte_bits: u64,
    pub page_mask: u64,
    pub ppn: u64,
}

pub struct Tlb {
    pub entries: [TlbEntry; 4096],
}

impl Tlb {
    pub fn new() -> Self {
        Self { entries: [TlbEntry::default(); 4096] }
    }

    #[inline]
    pub fn hash(vpn: u64) -> usize {
        (vpn as usize) & 0xFFF
    }

    // SFENCE.VMA x0, x0
    pub fn flush_all(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.valid = false;
        }
    }

    pub fn flush_asid(&mut self, asid: u16) {
        for entry in self.entries.iter_mut() {
            if !entry.is_global && entry.asid == asid {
                entry.valid = false;
            }
        }
    }

    pub fn flush_page(&mut self, vaddr: u64, asid: u16, ignore_asid: bool) {
        let vpn = vaddr >> 12;
        for entry in self.entries.iter_mut() {
            if entry.valid {
                let mask_vpn = vpn & !(entry.page_mask >> 12);
                let entry_vpn = entry.vpn & !(entry.page_mask >> 12);

                if mask_vpn == entry_vpn {
                    if ignore_asid || entry.is_global || entry.asid == asid {
                        entry.valid = false;
                    }
                }
            }
        }
    }
}
