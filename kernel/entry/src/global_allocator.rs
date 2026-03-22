use crate::early_heap::EarlyHeapAllocator;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct GlobalAllocator {
    active_allocator: AtomicUsize,
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator {
    active_allocator: AtomicUsize::new(0),
};

enum ActiveAllocator {
    EarlyHeap(&'static EarlyHeapAllocator),
    PagedAllocator(&'static kernel_core::allocator::Allocator),
}

const ALLOCATOR_KIND_MASK: usize = 0xF;
const EARLY_HEAP_KIND: usize = 1;
const PAGED_ALLOCATOR_KIND: usize = 2;

unsafe fn parse_active_allocator(active_allocator: usize) -> Option<ActiveAllocator> {
    if active_allocator == 0 {
        return None;
    }
    let allocator_kind = active_allocator & ALLOCATOR_KIND_MASK;
    let allocator_ptr = active_allocator & (!ALLOCATOR_KIND_MASK);
    match allocator_kind {
        EARLY_HEAP_KIND => Some(ActiveAllocator::EarlyHeap(unsafe {
            &*(allocator_ptr as *const EarlyHeapAllocator)
        })),
        PAGED_ALLOCATOR_KIND => Some(ActiveAllocator::PagedAllocator(unsafe {
            &*(allocator_ptr as *const kernel_core::allocator::Allocator)
        })),
        _ => todo!(),
    }
}

pub unsafe fn switch_to_early_heap(early_heap: &'static EarlyHeapAllocator) {
    let ptr = early_heap as *const EarlyHeapAllocator;
    assert_eq!(
        ptr.addr() % 16,
        0,
        "Early heap pointer {:p} is not 16-byte aligned!",
        ptr
    );
    let active_allocator: usize = ptr.addr() | EARLY_HEAP_KIND;

    GLOBAL_ALLOCATOR
        .active_allocator
        .store(active_allocator, Ordering::Release);
}

pub unsafe fn switch_to_paged_allocator(paged_allocator: &'static kernel_core::allocator::Allocator) {
    let ptr = paged_allocator as *const kernel_core::allocator::Allocator;
    assert_eq!(
        ptr.addr() % 16,
        0,
        "Paged allocator pointer {:p} is not 16-byte aligned!",
        ptr
    );
    let active_allocator: usize = ptr.addr() | PAGED_ALLOCATOR_KIND;

    GLOBAL_ALLOCATOR
        .active_allocator
        .store(active_allocator, Ordering::Release);
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator =
            unsafe { parse_active_allocator(self.active_allocator.load(Ordering::Acquire)) }
                .unwrap();
        match allocator {
            ActiveAllocator::EarlyHeap(early_heap) => unsafe { early_heap.alloc(layout) },
            ActiveAllocator::PagedAllocator(paged_allocator) => unsafe { paged_allocator.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocator =
            unsafe { parse_active_allocator(self.active_allocator.load(Ordering::Acquire)) }
                .unwrap();
        match allocator {
            ActiveAllocator::EarlyHeap(early_heap) => unsafe { early_heap.dealloc(ptr, layout) },
            ActiveAllocator::PagedAllocator(paged_allocator) => unsafe { paged_allocator.dealloc(ptr, layout) },
        }
    }
}
