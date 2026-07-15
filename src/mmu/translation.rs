#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Translation {
    pub physical_address: u64,
    pub translated: bool,

    /// Root page table physical address.
    pub root_page_table: u64,
}
