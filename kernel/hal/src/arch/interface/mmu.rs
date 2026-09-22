use atom_primitives::HhdmOffset;
use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct MMUMappingFlags: u32 {
        const PRESENT = 1 << 0;
        const WRITE = 1 << 1;
        const EXEC = 1 << 2;
        const USER = 1 << 3;
        const DEVICE = 1 << 4;
        const FRAMEBUFFER = 1 << 5;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PageSize {
    Normal,
    Large,
    Huge,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HhdmRegionKind {
    Normal,
    Device,
    Framebuffer,
}

#[derive(Copy, Clone, Debug)]
pub struct HhdmRegion {
    pub physical_start: usize,
    pub length: usize,
    pub kind: HhdmRegionKind,
}

#[derive(Copy, Clone, Debug)]
pub struct ExecutableAddress {
    pub physical_base: usize,
    pub virtual_base: usize,
    pub length: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct ExecutableRegion {
    pub virtual_start: usize,
    pub length: usize,
    pub writable: bool,
    pub executable: bool,
}

pub trait ArchMMU {
    type InitResult;

    /// Constructs new page tables containing the kernel executable and an HHDM.
    unsafe fn init(
        hhdm_offset: HhdmOffset,
        hhdm_regions: &mut dyn Iterator<Item = HhdmRegion>,
        executable: ExecutableAddress,
        executable_regions: &mut dyn Iterator<Item = ExecutableRegion>,
    ) -> Self::InitResult;

    /// Activates page tables built by [`ArchMMU::init`] and transfers control.
    unsafe fn activate(init_result: Self::InitResult, next_entrypoint: *const ()) -> !;

    unsafe fn map_page(
        root_page_table: usize,
        physical_address: usize,
        virtual_address: usize,
        page_size: PageSize,
        flags: MMUMappingFlags,
    );
}
