//! # Physical Memory Allocator
//!
//! This module provides functions for allocating and freeing physical page frames,
//! with all three possible sizes—normal (4 kiB), large (2 MiB), and huge (1 GiB).
//!
//! Note that it should be populated with the initial memory
//! using the usual free functions ([free_huge_page], [free_large_page], and [free_normal_page]).

use crate::memory_regions::{HUGE_PAGE_SIZE, LARGE_PAGE_SIZE, MemoryPage, NORMAL_PAGE_SIZE};
use atom_hal::sync::Spinlock;
use atom_primitives::HhdmOffset;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

static LAST_HUGE_PAGE: Spinlock<PageFramePtr> = Spinlock::new(PageFramePtr(null_mut()));
static LAST_LARGE_PAGE: Spinlock<PageFramePtr> = Spinlock::new(PageFramePtr(null_mut()));
static LAST_NORMAL_PAGE: Spinlock<PageFramePtr> = Spinlock::new(PageFramePtr(null_mut()));

static HHDM_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Represents an intrusive free linked-list node of page frames
struct PageFrameBlock {
    prev_page: PageFramePtr,
    phys_addr: usize,
}

#[derive(Debug, Copy, Clone)]
struct PageFramePtr(*mut PageFrameBlock);

unsafe impl Send for PageFramePtr {}

pub fn init(hhdm_offset: HhdmOffset) {
    HHDM_OFFSET
        .compare_exchange(0, *hhdm_offset, Ordering::Relaxed, Ordering::Relaxed)
        .expect("Physical memory allocator already initialized");
}

pub fn free_huge_page(page: MemoryPage<HUGE_PAGE_SIZE>) {
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    if hhdm_offset == 0 {
        panic!("Physical memory allocator not initialized");
    }
    let page_frame = (hhdm_offset + page.base_address) as *mut PageFrameBlock;
    let mut last_page = LAST_HUGE_PAGE.lock();
    unsafe {
        page_frame.write(PageFrameBlock {
            prev_page: *last_page,
            phys_addr: page.base_address,
        });
    }
    *last_page = PageFramePtr(page_frame);
}

pub fn free_large_page(page: MemoryPage<LARGE_PAGE_SIZE>) {
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    if hhdm_offset == 0 {
        panic!("Physical memory allocator not initialized");
    }
    let page_frame = (hhdm_offset + page.base_address) as *mut PageFrameBlock;
    let mut last_page = LAST_LARGE_PAGE.lock();
    unsafe {
        page_frame.write(PageFrameBlock {
            prev_page: *last_page,
            phys_addr: page.base_address,
        });
    }
    *last_page = PageFramePtr(page_frame);
}

pub fn free_normal_page(page: MemoryPage<NORMAL_PAGE_SIZE>) {
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    if hhdm_offset == 0 {
        panic!("Physical memory allocator not initialized");
    }
    let page_frame = (hhdm_offset + page.base_address) as *mut PageFrameBlock;
    let mut last_page = LAST_NORMAL_PAGE.lock();
    unsafe {
        page_frame.write(PageFrameBlock {
            prev_page: *last_page,
            phys_addr: page.base_address,
        });
    }
    *last_page = PageFramePtr(page_frame);
}

pub fn alloc_huge_page() -> Option<MemoryPage<HUGE_PAGE_SIZE>> {
    let mut last_page = LAST_HUGE_PAGE.lock();
    if last_page.0.is_null() {
        return None;
    }
    let page = last_page.0;
    *last_page = unsafe { last_page.0.read().prev_page };

    Some(MemoryPage {
        base_address: unsafe { page.read().phys_addr },
    })
}

pub fn alloc_large_page() -> Option<MemoryPage<LARGE_PAGE_SIZE>> {
    let mut last_page = LAST_LARGE_PAGE.lock();
    if last_page.0.is_null() {
        let huge_page = alloc_huge_page()?;
        let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
        if hhdm_offset == 0 {
            return None;
        }
        for page_idx in 0..(HUGE_PAGE_SIZE / LARGE_PAGE_SIZE) {
            let page_addr = huge_page.base_address + page_idx * LARGE_PAGE_SIZE;
            let page_ptr = (page_addr + hhdm_offset) as *mut PageFrameBlock;
            unsafe {
                page_ptr.write(PageFrameBlock {
                    prev_page: *last_page,
                    phys_addr: page_addr,
                });
            }
            *last_page = PageFramePtr(page_ptr);
        }
    }
    let page = last_page.0;
    *last_page = unsafe { last_page.0.read().prev_page };

    Some(MemoryPage {
        base_address: unsafe { page.read().phys_addr },
    })
}

pub fn alloc_normal_page() -> Option<MemoryPage<NORMAL_PAGE_SIZE>> {
    let mut last_page = LAST_NORMAL_PAGE.lock();
    if last_page.0.is_null() {
        let large_page = alloc_large_page()?;
        let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
        if hhdm_offset == 0 {
            return None;
        }
        for page_idx in 0..(LARGE_PAGE_SIZE / NORMAL_PAGE_SIZE) {
            let page_addr = large_page.base_address + page_idx * NORMAL_PAGE_SIZE;
            let page_ptr = (page_addr + hhdm_offset) as *mut PageFrameBlock;
            unsafe {
                page_ptr.write(PageFrameBlock {
                    prev_page: *last_page,
                    phys_addr: page_addr,
                });
            }
            *last_page = PageFramePtr(page_ptr);
        }
    }
    let page = last_page.0;
    *last_page = unsafe { last_page.0.read().prev_page };

    Some(MemoryPage {
        base_address: unsafe { page.read().phys_addr },
    })
}
