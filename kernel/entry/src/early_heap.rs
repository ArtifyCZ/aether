use core::alloc::{GlobalAlloc, Layout};
use core::mem::zeroed;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

const EARLY_HEAP_SIZE: usize = 0x1000;
#[repr(align(16))]
struct EarlyHeapMemory([u8; EARLY_HEAP_SIZE]);

static mut MEMORY: EarlyHeapMemory = EarlyHeapMemory(unsafe { zeroed() });

const MEMORY_END: *mut u8 = unsafe { (&raw mut MEMORY.0 as *mut u8).add(EARLY_HEAP_SIZE) };

pub struct EarlyHeapAllocator {
    next_available_addr: AtomicUsize,
}

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}


pub unsafe fn init() -> &'static EarlyHeapAllocator {
    unsafe {
        let early_heap_mem_ptr = &raw mut MEMORY.0 as *mut u8;
        let next_available_addr = early_heap_mem_ptr.add(size_of::<EarlyHeapAllocator>());
        let early_heap_allocator_ptr = early_heap_mem_ptr as *mut EarlyHeapAllocator;
        early_heap_allocator_ptr.write(EarlyHeapAllocator{
            next_available_addr: AtomicUsize::new(next_available_addr.addr()),
        });

        &*early_heap_allocator_ptr
    }
}

unsafe impl GlobalAlloc for EarlyHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        loop {
            let available_addr = self.next_available_addr.load(Ordering::Acquire);
            let possible_addr = align_up(available_addr, align);
            let next_available_addr = possible_addr + size;
            if next_available_addr >= MEMORY_END.addr() {
                return null_mut();
            }
            if self.next_available_addr.compare_exchange(available_addr, next_available_addr, Ordering::Release, Ordering::Relaxed).is_ok() {
                return possible_addr as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Bump allocator - dealloc is no-op
    }
}
