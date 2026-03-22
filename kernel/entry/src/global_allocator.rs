use crate::early_heap::EarlyHeapAllocator;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

#[repr(align(16))]
pub struct GlobalAllocator {
    active_allocator: AtomicUsize,
}

#[unsafe(no_mangle)]
#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator {
    active_allocator: AtomicUsize::new(0),
};

const ALLOCATOR_KIND_MASK: usize = 0xF;
const EARLY_HEAP_KIND: usize = 1;
const PAGED_ALLOCATOR_KIND: usize = 2;

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

pub unsafe fn switch_to_paged_allocator(
    paged_allocator: &'static kernel_core::allocator::Allocator,
) {
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
        let allocator = self.active_allocator.load(Ordering::Acquire);
        let addr = allocator & (!ALLOCATOR_KIND_MASK);
        let kind = allocator & ALLOCATOR_KIND_MASK;
        match kind {
            EARLY_HEAP_KIND => unsafe { (&*(addr as *const EarlyHeapAllocator)).alloc(layout) },
            PAGED_ALLOCATOR_KIND => unsafe {
                (&*(addr as *const kernel_core::allocator::Allocator)).alloc(layout)
            },
            _ => unsafe {
                core::arch::asm!("int3", options(noreturn))
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocator = self.active_allocator.load(Ordering::Acquire);
        let addr = allocator & (!ALLOCATOR_KIND_MASK);
        let kind = allocator & ALLOCATOR_KIND_MASK;
        match kind {
            EARLY_HEAP_KIND => unsafe {
                (&*(addr as *const EarlyHeapAllocator)).dealloc(ptr, layout)
            },
            PAGED_ALLOCATOR_KIND => unsafe {
                (&*(addr as *const kernel_core::allocator::Allocator)).dealloc(ptr, layout)
            },
            _ => {}
        }
    }
}
