use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ProxyAllocator(AtomicUsize);

const ALLOC_ADDR_MASK: usize = !0xF; // everything but 4 bits (which are reserved as the kind marker)
const ALLOC_KIND_MASK: usize = 0xF; // 4 bits
const ALLOC_KIND_PAGED: usize = 2;

impl ProxyAllocator {
    pub const unsafe fn init() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub unsafe fn switch_to_paged_allocator(
        &self,
        allocator: *const kernel_core::allocator::Allocator,
    ) {
        assert_eq!(
            allocator.addr() % 16,
            0,
            "Allocator ({:p}) must be 16-byte aligned!",
            allocator
        );
        let value = allocator.addr() | ALLOC_KIND_PAGED;
        self.0.store(value, Ordering::Release);
    }
}

unsafe impl GlobalAlloc for ProxyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let value = self.0.load(Ordering::Acquire);
        let addr = value & ALLOC_ADDR_MASK;
        let kind = value & ALLOC_KIND_MASK;
        match kind {
            ALLOC_KIND_PAGED => {
                let allocator = addr as *const kernel_core::allocator::Allocator;
                let Some(allocator) = (unsafe { allocator.as_ref() }) else {
                    return null_mut();
                };
                unsafe { allocator.alloc(layout) }
            }
            _ => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let value = self.0.load(Ordering::Acquire);
        let addr = value & ALLOC_ADDR_MASK;
        let kind = value & ALLOC_KIND_MASK;
        match kind {
            ALLOC_KIND_PAGED => {
                let allocator = addr as *const kernel_core::allocator::Allocator;
                let Some(allocator) = (unsafe { allocator.as_ref() }) else {
                    return;
                };
                unsafe { allocator.dealloc(ptr, layout) }
            }
            _ => {}
        }
    }
}
