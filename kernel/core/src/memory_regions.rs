#[derive(Debug)]
pub struct MemoryRegion {
    pub base_address: usize,
    pub length: usize,
    pub kind: MemoryRegionKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionKind {
    /// Usable with no strings or requirements attached
    Usable,
    /// Can be used—reclaimed, but anything to be kept should be filtered-out first
    Reclaimable,
    /// Parts can theoretically be reclaimed, contains the kernel and modules (e.g., initrd)
    ExecutableAndModules,
    /// Should not be used
    Reserved,
    /// Should not be used, the memory is not reliable
    BadMemory,
}
