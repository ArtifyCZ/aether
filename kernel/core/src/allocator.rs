use crate::platform::memory_layout::{KERNEL_HEAP_BASE, PAGE_FRAME_SIZE};
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;
use crate::platform::virtual_page_address::VirtualPageAddress;
use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::MaybeUninit;
use core::ptr::null_mut;
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

pub static mut ALLOCATOR_STORAGE: MaybeUninit<Allocator> = MaybeUninit::uninit();

#[repr(align(16))]
struct AllocatorInner {
    next_available_alloc_vaddr: usize,
    mapped_memory_end_vaddr: usize,
}

const INITIAL_MAPPED_PAGES_COUNT: usize = 0x1000;

impl Allocator {
    pub unsafe fn init() -> &'static Self {
        let mut mapped_memory_end_vaddr = VirtualPageAddress::new(KERNEL_HEAP_BASE).unwrap();
        for _ in 0..INITIAL_MAPPED_PAGES_COUNT {
            unsafe {
                let context = VirtualMemoryManagerContext::get_kernel_context();
                let phys = PhysicalMemoryManager::alloc_frame().unwrap();
                context.map_page(mapped_memory_end_vaddr, phys, VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE).unwrap();
                mapped_memory_end_vaddr = mapped_memory_end_vaddr.next_page();
            }
        }
        #[allow(static_mut_refs)]
        unsafe {
            let allocator = Allocator(InterruptSafeSpinLock::new(AllocatorInner {
                next_available_alloc_vaddr: KERNEL_HEAP_BASE,
                mapped_memory_end_vaddr: mapped_memory_end_vaddr.inner(),
            }));
            ALLOCATOR_STORAGE.write(allocator);
            ALLOCATOR_STORAGE.assume_init_ref()
        }
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
                return null_mut();
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
