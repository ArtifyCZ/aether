use crate::platform::memory_layout::{KERNEL_HEAP_BASE, PAGE_FRAME_SIZE};
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::platform::virtual_page_address::VirtualPageAddress;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};
use kernel_hal::mmu::VirtualMemoryMappingFlags;

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

const STATIC_HEAP_SIZE: usize = 0x8_0000; // 512 KiB

#[repr(align(16))]
struct StaticHeapMemory([u8; STATIC_HEAP_SIZE]);

static mut STATIC_HEAP_MEMORY: StaticHeapMemory = StaticHeapMemory([0; STATIC_HEAP_SIZE]);

#[repr(align(16))]
pub struct Allocator {
    static_heap_next_available_idx: AtomicUsize,
}

impl Allocator {
    pub const unsafe fn init() -> Self {
        Allocator {
            static_heap_next_available_idx: AtomicUsize::new(0),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        loop {
            let next_available_idx = self.static_heap_next_available_idx.load(Ordering::Acquire);
            let static_start = align_up(next_available_idx, align);
            let static_end = static_start + size;
            if static_end >= STATIC_HEAP_SIZE {
                return null_mut(); // Out of static heap memory
            }

            if self
                .static_heap_next_available_idx
                .compare_exchange(
                    next_available_idx,
                    static_end,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_err()
            {
                continue; // Retry if the CAS failed (another thread beat us to it)
            }

            let ptr = (unsafe { &raw mut STATIC_HEAP_MEMORY.0 as usize } + static_start) as *mut u8;
            assert_eq!(
                ptr.addr() % align,
                0,
                "Static heap pointer {:p} is not aligned to {}!",
                ptr,
                align,
            );
            return ptr;
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op.
    }
}
