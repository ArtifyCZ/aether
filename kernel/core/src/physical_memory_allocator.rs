use crate::memory_regions::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, MemoryPage};
use atom_primitives::HhdmOffset;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

static LAST_HUGE_PAGE: AtomicPtr<PageFrameBlock> = AtomicPtr::new(null_mut());
static LAST_LARGE_PAGE: AtomicPtr<PageFrameBlock> = AtomicPtr::new(null_mut());

static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Represents an intrusive free linked-list node of page frames
struct PageFrameBlock {
    prev_page: *mut PageFrameBlock,
    phys_addr: usize,
}

pub fn init(hhdm_offset: HhdmOffset) {
    HHDM_OFFSET
        .compare_exchange(0, *hhdm_offset, Ordering::Relaxed, Ordering::Relaxed)
        .expect("Physical memory allocator already initialized");
}

pub fn free_huge_page(page: MemoryPage<HUGE_PAGE_SIZE>) {
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    if hhdm_offset == 0 {
        panic!("Physical memory allocator not initialized")
    }
    let page_frame = (hhdm_offset + page.base_address) as *mut PageFrameBlock;
    loop {
        let last_page = LAST_HUGE_PAGE.load(Ordering::Acquire);
        let block = PageFrameBlock {
            prev_page: last_page,
            phys_addr: page.base_address,
        };
        unsafe {
            page_frame.write(block);
        }
        if LAST_HUGE_PAGE
            .compare_exchange(last_page, page_frame, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}

pub fn free_large_page(page: MemoryPage<LARGE_PAGE_SIZE>) {
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    if hhdm_offset == 0 {
        panic!("Physical memory allocator not initialized");
    }
    let page_frame = (hhdm_offset + page.base_address) as *mut PageFrameBlock;
    loop {
        let last_page = LAST_LARGE_PAGE.load(Ordering::Acquire);
        let block = PageFrameBlock {
            prev_page: last_page,
            phys_addr: page.base_address,
        };
        unsafe {
            page_frame.write(block);
        }
        if LAST_LARGE_PAGE
            .compare_exchange(last_page, page_frame, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}
