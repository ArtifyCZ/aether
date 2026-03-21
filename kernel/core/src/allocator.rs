use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;
use crate::platform::memory_layout::{KERNEL_HEAP_BASE, PAGE_FRAME_SIZE};
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::platform::virtual_page_address::VirtualPageAddress;
use core::alloc::{GlobalAlloc, Layout};
use kernel_hal::mmu::VirtualMemoryMappingFlags;

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

#[repr(align(16))]
struct EarlyHeap([u8; EARLY_HEAP_SIZE]);

const EARLY_HEAP_SIZE: usize = 0x4_0000;
static mut EARLY_HEAP_MEMORY: EarlyHeap = EarlyHeap([0; EARLY_HEAP_SIZE]);

pub struct Allocator(InterruptSafeSpinLock<AllocatorInner>);

struct AllocatorInner {
    early_heap_next_available_idx: usize,
    next_available_virt_addr: usize,
    /// The highest virtual address currently backed by physical frames
    current_heap_limit: usize,
}

pub static GLOBAL_ALLOCATOR: Allocator = Allocator(InterruptSafeSpinLock::new(AllocatorInner {
    early_heap_next_available_idx: 0,
    next_available_virt_addr: KERNEL_HEAP_BASE,
    current_heap_limit: KERNEL_HEAP_BASE,
}));

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Initial lock to check status
        {
            let mut inner = self.0.lock();

            // Try Early Heap First
            let early_start = align_up(inner.early_heap_next_available_idx, align);
            if early_start + size <= EARLY_HEAP_SIZE {
                inner.early_heap_next_available_idx = early_start + size;
                let ptr = (unsafe { &raw mut EARLY_HEAP_MEMORY.0 as usize } + early_start) as *mut u8;
                assert_eq!(ptr.addr() % align, 0, "Early heap pointer {:p} is not aligned to {}!", ptr, align);
                return ptr;
            }

            // Paged Heap check
            let vaddr_start = align_up(inner.next_available_virt_addr, align);
            let required_limit = vaddr_start + size;

            // If we have enough mapped memory, just bump and return
            if required_limit <= inner.current_heap_limit {
                inner.next_available_virt_addr = required_limit;
                let ptr = vaddr_start as *mut u8;
                assert_eq!(ptr.addr() % align, 0, "Paged heap pointer {:p} is not aligned to {}!", ptr, align);
                return ptr;
            }
        }

        // If we reach here, we need to map more pages.
        // We do NOT hold the lock here. Interrupts are ENABLED.
        self.expand_and_alloc(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op.
    }
}

impl Allocator {
    /// Handles mapping more physical memory. Called without the lock held.
    unsafe fn expand_and_alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Calculate how many pages we need to add to satisfy this request plus a buffer
        let pages_to_alloc = (align_up(size, PAGE_FRAME_SIZE) / PAGE_FRAME_SIZE) + 8;

        // We must re-lock briefly to see where the current limit is
        let mapping_start_vaddr = self.0.lock().current_heap_limit;
        let vmm = VirtualMemoryManagerContext::get_kernel_context();

        for i in 0..pages_to_alloc {
            let phys = PhysicalMemoryManager::alloc_frame()
                .expect("OOM: Failed to allocate physical frame for kernel heap");

            let vaddr = mapping_start_vaddr + (i * PAGE_FRAME_SIZE);
            let vpage = VirtualPageAddress::new(vaddr).unwrap();

            vmm.map_page(
                vpage,
                phys,
                VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE,
            )
            .expect("Failed to map kernel heap page");
        }

        // Re-acquire lock to update the limit and perform the actual bump
        let mut inner = self.0.lock();

        // Update the limit since we successfully mapped pages
        let new_limit = mapping_start_vaddr + (pages_to_alloc * PAGE_FRAME_SIZE);
        if new_limit > inner.current_heap_limit {
            inner.current_heap_limit = new_limit;
        }

        let vaddr_start = align_up(inner.next_available_virt_addr, align);
        inner.next_available_virt_addr = vaddr_start + size;

        let ptr = vaddr_start as *mut u8;
        assert_eq!(ptr.addr() % align, 0, "Paged heap pointer {:p} is not aligned to {}!", ptr, align);

        ptr
    }
}
