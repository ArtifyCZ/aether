use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
    sync::atomic::{AtomicUsize, Ordering},
};

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

const STATIC_HEAP_SIZE: usize = 0x1_0000; // 64 KiB

#[repr(align(16))]
struct StaticHeapMemory([u8; STATIC_HEAP_SIZE]);

static mut STATIC_HEAP_MEMORY: StaticHeapMemory = StaticHeapMemory([0; STATIC_HEAP_SIZE]);

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator {
    static_heap_next_addr: AtomicUsize::new(0),
};

struct Allocator {
    static_heap_next_addr: AtomicUsize,
}

#[allow(static_mut_refs)]
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let static_heap_mem_addr = unsafe { STATIC_HEAP_MEMORY.0.as_ptr().addr() };
        assert_eq!(
            static_heap_mem_addr % 16,
            0,
            "Static heap memory must be 16-byte aligned"
        );
        let static_heap_mem_end_addr = static_heap_mem_addr + STATIC_HEAP_SIZE;

        loop {
            let allocated_addr_end = self.static_heap_next_addr.load(Ordering::Acquire);

            let start_addr = if allocated_addr_end != 0 {
                allocated_addr_end
            } else {
                static_heap_mem_addr
            };
            let start_addr = align_up(start_addr, align);
            let end_addr = start_addr + size;

            if end_addr >= static_heap_mem_end_addr {
                // return null_mut();
                // @TODO: Return a null pointer, as Rust expects to be done.
                loop {}
            }

            if self
                .static_heap_next_addr
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
