use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_address::VirtualAddress;
use crate::platform::virtual_address_allocator::VirtualAddressAllocator;
use crate::platform::virtual_memory_manager_context::{
    VirtualMemoryManagerContext, VirtualMemoryMappingFlags,
};
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::spin_lock::SpinLock;
use core::alloc::{GlobalAlloc, Layout};
use core::ops::Add;

static mut NEXT_AVAILABLE_VIRTUAL_ADDRESS: Option<VirtualAddress> = None;
static mut LAST_MAPPED_VIRTUAL_PAGE: Option<VirtualPageAddress> = None;
static mut VMM_CONTEXT: Option<VirtualMemoryManagerContext> = None;
static HEAP_LOCK: SpinLock = SpinLock::new();

pub struct Allocator;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Allocator = Allocator;

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;
    fn add(self, rhs: usize) -> Self::Output {
        VirtualAddress::new(self.inner() + rhs).unwrap()
    }
}

#[allow(static_mut_refs)]
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // pad_to_align ensures the size is a multiple of the alignment,
        // but we still need to ensure the START address is aligned.
        let layout = layout.pad_to_align();
        let _lock = HEAP_LOCK.lock();

        let vmm_context = if let Some(ref vmm_context) = VMM_CONTEXT {
            vmm_context
        } else {
            let vmm_context = VirtualMemoryManagerContext::get_kernel_context();
            VMM_CONTEXT = Some(vmm_context);
            VMM_CONTEXT.as_ref().unwrap()
        };

        loop {
            if let Some(mut next_addr) = NEXT_AVAILABLE_VIRTUAL_ADDRESS {
                // --- ALIGNMENT LOGIC ---
                // Calculate the padding needed to satisfy layout.align()
                let addr_val = next_addr.inner();
                let align = layout.align();
                let aligned_addr = (addr_val + align - 1) & !(align - 1);

                // Temporarily update next_addr to the aligned position for the bounds check
                next_addr = VirtualAddress::new(aligned_addr).unwrap();

                if let Some(last_page) = LAST_MAPPED_VIRTUAL_PAGE {
                    if next_addr.inner() + layout.size() <= last_page.end().inner() {
                        // Success! Update global state and return
                        let ptr = next_addr.inner() as *mut u8;
                        NEXT_AVAILABLE_VIRTUAL_ADDRESS = Some(next_addr + layout.size());
                        return ptr;
                    }
                }
            }

            // If we reach here, we either have no NEXT_AVAILABLE or no space left.
            // Map a new page.
            let new_frame = match PhysicalMemoryManager::alloc_frame() {
                Ok(frame) => frame,
                Err(_err) => {
                    panic!("OOM: Physical frame allocation failed\n");
                }
            };

            let next_virt_page_addr = if let Some(last_page) = LAST_MAPPED_VIRTUAL_PAGE {
                last_page.next_page()
            } else {
                // Initial heap setup: allocate a large virtual range
                VirtualAddressAllocator::alloc_range(PAGE_FRAME_SIZE * 1024 * 1024)
            };

            // Ensure not already mapped
            if vmm_context.translate(next_virt_page_addr).unwrap().is_some() {
                panic!("Allocator Error: Virtual page already mapped\n");
            }

            if vmm_context.map_page(
                next_virt_page_addr,
                new_frame,
                VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE,
            ).is_err() {
                panic!("Allocator Error: Failed to map page\n");
            }

            LAST_MAPPED_VIRTUAL_PAGE = Some(next_virt_page_addr);
            if NEXT_AVAILABLE_VIRTUAL_ADDRESS.is_none() {
                NEXT_AVAILABLE_VIRTUAL_ADDRESS = Some(next_virt_page_addr.start());
            }

            // Loop repeats to re-check the space with the newly mapped page
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op until we implement a proper heap.
    }
}
