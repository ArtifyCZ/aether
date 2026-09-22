use crate::arch::current::mmu::ArchMMUImpl;
use crate::arch::interface::mmu::ArchMMU;
use atom_primitives::HhdmOffset;

pub use crate::arch::interface::mmu::{
    ExecutableAddress, ExecutableRegion, HhdmRegion, HhdmRegionKind,
};

pub unsafe fn init(
    hhdm_offset: HhdmOffset,
    hhdm_regions: impl IntoIterator<Item = HhdmRegion>,
    executable: ExecutableAddress,
    executable_regions: impl IntoIterator<Item = ExecutableRegion>,
    next_entrypoint: *const (),
) -> ! {
    let mut hhdm_regions = hhdm_regions.into_iter();
    let mut executable_regions = executable_regions.into_iter();
    let init_result = unsafe {
        ArchMMUImpl::init(
            hhdm_offset,
            &mut hhdm_regions,
            executable,
            &mut executable_regions,
        )
    };
    unsafe { ArchMMUImpl::activate(init_result, next_entrypoint) }
}
