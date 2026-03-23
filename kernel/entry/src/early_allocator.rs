use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

const EARLY_HEAP_SIZE: usize = 0x10_0000; // 64 kiB

#[repr(align(16))]
pub struct EarlyAllocator {
    allocated_addr_end: AtomicUsize,
    memory: EarlyHeapMemoryPtr,
}

struct EarlyHeapMemoryPtr(*mut EarlyHeapMemory);

unsafe impl Send for EarlyHeapMemoryPtr {}

unsafe impl Sync for EarlyHeapMemoryPtr {}

#[repr(align(16))]
struct EarlyHeapMemory([u8; EARLY_HEAP_SIZE]);

static INSTANCE: EarlyAllocator = EarlyAllocator {
    allocated_addr_end: AtomicUsize::new(0),
    memory: EarlyHeapMemoryPtr(unsafe { &raw mut MEMORY }),
};

static mut MEMORY: EarlyHeapMemory = EarlyHeapMemory([0; EARLY_HEAP_SIZE]);

impl EarlyAllocator {
    pub const unsafe fn init() -> EarlyAllocator {
        EarlyAllocator {
            allocated_addr_end: AtomicUsize::new(0),
            memory: EarlyHeapMemoryPtr(unsafe { &raw mut MEMORY }),
        }
    }
}

unsafe impl GlobalAlloc for EarlyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let early_heap_mem_addr = self.memory.0.addr();
        let early_alloc_end_addr = early_heap_mem_addr + EARLY_HEAP_SIZE;

        loop {
            let allocated_addr_end = self.allocated_addr_end.load(Ordering::Acquire);

            let start_addr = if allocated_addr_end != 0 {
                allocated_addr_end
            } else {
                early_heap_mem_addr
            };
            let start_addr = align_up(start_addr, align);
            let end_addr = start_addr + size;

            if end_addr >= early_alloc_end_addr {
                return null_mut();
            }

            if self
                .allocated_addr_end
                .compare_exchange(
                    allocated_addr_end,
                    end_addr,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return start_addr as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Bump allocator - dealloc is no-op
    }
}
