use crate::platform::memory_layout::{KERNEL_HEAP_BASE, PAGE_FRAME_SIZE};
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::platform::virtual_page_address::VirtualPageAddress;
use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel_hal::mmu::VirtualMemoryMappingFlags;
use crate::interrupt_safe_spin_lock::InterruptSafeSpinLock;

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

pub struct Allocator(InterruptSafeSpinLock<AllocatorInner>);

#[repr(align(16))]
struct AllocatorInner {
    next_available_alloc_vaddr: usize,
    mapped_memory_end_vaddr: usize,
}

impl Allocator {
    pub unsafe fn init() -> &'static Self {
        Box::leak(Box::new(Allocator(InterruptSafeSpinLock::new(AllocatorInner {
            next_available_alloc_vaddr: KERNEL_HEAP_BASE,
            mapped_memory_end_vaddr: KERNEL_HEAP_BASE,
        }))))
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let mut inner = self.0.lock();
        let start = align_up(inner.next_available_alloc_vaddr, align);
        let next_available_vaddr = start + size;
        {
            let end = inner.mapped_memory_end_vaddr;
            if end <= next_available_vaddr {
                let pages_count = align_up(size, PAGE_FRAME_SIZE) + 8;
                inner.expand(pages_count);
            }
        }

        let end = inner.mapped_memory_end_vaddr;
        assert!(end > next_available_vaddr, "It should have been expanded!");

        inner.next_available_alloc_vaddr = next_available_vaddr;

        start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op.
    }
}

impl AllocatorInner {
    unsafe fn expand(&mut self, pages_count: usize) {
        let base = self.mapped_memory_end_vaddr;
        for i in 0..pages_count {
            let page_vaddr = VirtualPageAddress::new(base + i * PAGE_FRAME_SIZE).unwrap();
            let phys = unsafe { PhysicalMemoryManager::alloc_frame() }.unwrap();
            let context = unsafe { VirtualMemoryManagerContext::get_kernel_context() };
            unsafe { context.map_page(page_vaddr, phys, VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE) }.unwrap();
        }
        self.mapped_memory_end_vaddr = base + pages_count * PAGE_FRAME_SIZE;
    }
}
