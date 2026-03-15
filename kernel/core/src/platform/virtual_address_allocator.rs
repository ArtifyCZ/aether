use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::virtual_page_address::VirtualPageAddress;

pub struct VirtualAddressAllocator;

fn align_up(v: usize, a: usize) -> usize {
    let mask = !(a - 1);
    (v + (a - 1)) & mask
}

static mut NEXT_RANGE_START: usize = 0;

#[unsafe(no_mangle)]
unsafe extern "C" fn vaa_alloc_range(size: usize) -> usize {
    let size = align_up(size, PAGE_FRAME_SIZE);
    unsafe {
        let ret = NEXT_RANGE_START;
        NEXT_RANGE_START += size + PAGE_FRAME_SIZE;
        ret
    }
}

impl VirtualAddressAllocator {
    pub unsafe fn init() {
        unsafe {
            NEXT_RANGE_START = 0xFFFF_C000_0000_0000;
        }
    }

    pub unsafe fn alloc_range(size: usize) -> VirtualPageAddress {
        unsafe { vaa_alloc_range(size) }
            .try_into()
            .unwrap()
    }
}
