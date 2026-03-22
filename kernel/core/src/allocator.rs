use crate::platform::memory_layout::{KERNEL_HEAP_BASE, PAGE_FRAME_SIZE};
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::platform::virtual_page_address::VirtualPageAddress;
use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use kernel_hal::mmu::VirtualMemoryMappingFlags;

fn align_up(v: usize, a: usize) -> usize {
    if a == 0 {
        return v;
    }
    let mask = a - 1;
    (v + mask) & !mask
}

#[repr(align(16))]
pub struct Allocator {
    next_available_alloc_vaddr: AtomicUsize,
    mapped_memory_end_vaddr: AtomicUsize,
    mapping_additional_memory: AtomicBool,
}

impl Allocator {
    pub unsafe fn init() -> &'static Self {
        Box::leak(Box::new(Allocator {
            next_available_alloc_vaddr: AtomicUsize::new(KERNEL_HEAP_BASE),
            mapped_memory_end_vaddr: AtomicUsize::new(KERNEL_HEAP_BASE),
            mapping_additional_memory: AtomicBool::new(false),
        }))
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        loop {
            let start = self.next_available_alloc_vaddr.load(Ordering::Acquire);
            let end = self.mapped_memory_end_vaddr.load(Ordering::Relaxed);
            let aligned_start = align_up(start, align);
            let next_available_vaddr = aligned_start + size;
            if next_available_vaddr >= end {
                // Not enough mapped memory
                while self.mapping_additional_memory.load(Ordering::Acquire) {
                    core::hint::spin_loop();
                }
                let pages_count = align_up(size, PAGE_FRAME_SIZE) + 2;
                unsafe {
                    self.expand(pages_count);
                }
                self.mapping_additional_memory.store(false, Ordering::Release);
                continue;
            }
            if self
                .next_available_alloc_vaddr
                .compare_exchange(
                    start,
                    next_available_vaddr,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return aligned_start as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op.
    }
}

impl Allocator {
    unsafe fn expand(&self, pages_count: usize) {
        let base = self.mapped_memory_end_vaddr.load(Ordering::Acquire);
        for i in 0..pages_count {
            let page_vaddr = VirtualPageAddress::new(base + i * PAGE_FRAME_SIZE).unwrap();
            let phys = unsafe { PhysicalMemoryManager::alloc_frame() }.unwrap();
            let context = unsafe { VirtualMemoryManagerContext::get_kernel_context() };
            unsafe { context.map_page(page_vaddr, phys, VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE) }.unwrap();
        }
        self.mapped_memory_end_vaddr.store(base + pages_count * PAGE_FRAME_SIZE, Ordering::Release);
    }
}
