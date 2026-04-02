use core::ffi::CStr;
use core::str::FromStr;

use crate::elf::Elf;
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_address_allocator::VirtualAddressAllocator;
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::platform::{
    modules::Modules, virtual_memory_manager_context::VirtualMemoryManagerContext,
};
use crate::println;
use crate::scheduler::Scheduler;
use crate::tarball_parsing::parse_tarball_archive;
use crate::task_registry::TaskSpec;
use alloc::{ffi::CString, sync::Arc};
use kernel_hal::mmu::VirtualMemoryMappingFlags;

fn load_init_into_memory(
    init_elf: &[u8],
    elf: &Elf,
    init_ctx: &VirtualMemoryManagerContext,
) -> usize {
    let entrypoint_vaddr = unsafe { elf.load(init_ctx, init_elf.as_ptr()) }.unwrap();
    entrypoint_vaddr
}

fn load_initrd_into_memory(
    initrd: &[u8],
    init_ctx: &VirtualMemoryManagerContext,
) -> (usize, usize) {
    const INITRD_VADDR: usize = 0x2FFFFFF00000usize;
    // Map the pages for initrd into the virtual memory and copy the data
    let initrd_size = initrd.len();
    let num_pages = (initrd_size + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;
    for i in 0..num_pages {
        unsafe {
            let page_vaddr = INITRD_VADDR + i * PAGE_FRAME_SIZE;
            let page_phys = PhysicalMemoryManager::alloc_frame().unwrap();
            let kernel_vaddr = VirtualAddressAllocator::alloc_range(PAGE_FRAME_SIZE);
            init_ctx
                .map_page(
                    VirtualPageAddress::new(page_vaddr).unwrap(),
                    page_phys,
                    VirtualMemoryMappingFlags::PRESENT
                        | VirtualMemoryMappingFlags::USER
                        | VirtualMemoryMappingFlags::WRITE,
                )
                .unwrap();
            VirtualMemoryManagerContext::get_kernel_context()
                .map_page(
                    kernel_vaddr,
                    page_phys,
                    VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE,
                )
                .unwrap();
            // Copy the data from the initrd module to the mapped page
            let src_ptr = initrd.as_ptr().add(i * PAGE_FRAME_SIZE);
            let dst_ptr = kernel_vaddr.start().inner() as *mut u8;
            let remaining = initrd_size - (i * PAGE_FRAME_SIZE);
            let to_copy = core::cmp::min(PAGE_FRAME_SIZE, remaining);
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, to_copy);
            // Zero the rest of the page if it's a partial copy
            if to_copy < PAGE_FRAME_SIZE {
                core::ptr::write_bytes(dst_ptr.add(to_copy), 0, PAGE_FRAME_SIZE - to_copy);
            }
        }
    }

    (INITRD_VADDR, initrd_size)
}

fn load_boot_info_into_memory(
    init_ctx: &VirtualMemoryManagerContext,
    initrd_start: usize,
    initrd_size: usize,
) -> u64 {
    const BOOT_INFO_VADDR: usize = 0x7FFFFFE00000usize; // arbitrary high virtual address for boot info
    const BOOT_INFO_SIZE: usize = core::mem::size_of::<init_contract_rust::boot_info>();
    let kernel_vaddr = unsafe { VirtualAddressAllocator::alloc_range(BOOT_INFO_SIZE) };
    const NUM_PAGES: usize = (BOOT_INFO_SIZE + PAGE_FRAME_SIZE - 1) / PAGE_FRAME_SIZE;
    for i in 0..NUM_PAGES {
        unsafe {
            let page_vaddr = BOOT_INFO_VADDR + i * PAGE_FRAME_SIZE;
            let page_phys = PhysicalMemoryManager::alloc_frame().unwrap();
            init_ctx
                .map_page(
                    VirtualPageAddress::new(page_vaddr).unwrap(),
                    page_phys,
                    VirtualMemoryMappingFlags::PRESENT
                        | VirtualMemoryMappingFlags::USER
                        | VirtualMemoryMappingFlags::WRITE,
                )
                .unwrap();
            VirtualMemoryManagerContext::get_kernel_context().map_page(
                kernel_vaddr,
                page_phys,
                VirtualMemoryMappingFlags::PRESENT | VirtualMemoryMappingFlags::WRITE,
            );
            core::ptr::write_bytes(kernel_vaddr.start().inner() as *mut u8, 0, PAGE_FRAME_SIZE);
        }
    }

    unsafe {
        let boot_info = kernel_vaddr.start().inner() as *mut init_contract_rust::boot_info;
        boot_info.write(init_contract_rust::boot_info {
            initrd_start: initrd_start as *mut core::ffi::c_void,
            initrd_size,
        });
    }

    BOOT_INFO_VADDR as u64
}

fn allocate_init_stack(init_ctx: &VirtualMemoryManagerContext) -> usize {
    const INIT_STACK_TOP_VADDR: usize = 0x7FFFFFFFF000usize;
    for i in 0..4 {
        // allocate 4 pages as stack
        let page_vaddr = INIT_STACK_TOP_VADDR - (i + 1) * PAGE_FRAME_SIZE;
        unsafe {
            let page_phys = PhysicalMemoryManager::alloc_frame().unwrap();
            init_ctx
                .map_page(
                    VirtualPageAddress::new(page_vaddr).unwrap(),
                    page_phys,
                    VirtualMemoryMappingFlags::PRESENT
                        | VirtualMemoryMappingFlags::USER
                        | VirtualMemoryMappingFlags::WRITE,
                )
                .unwrap();
        }
    }

    INIT_STACK_TOP_VADDR
}

pub fn spawn_init_process(init_program_name: &str, elf: &Elf, scheduler: &Scheduler) {
    let initrd = unsafe { Modules::find(c"initrd") }.expect("Initrd module not found");
    let initrd_tarball = parse_tarball_archive(initrd).unwrap();
    let init_elf = initrd_tarball
        .iter()
        .find(|h| {
            h.name
                .to_str()
                .map(|s| {
                    s == init_program_name
                        .split_once('/')
                        .map(|(_, n)| n)
                        .unwrap_or(init_program_name)
                })
                .unwrap_or(false)
        })
        .expect("Could not find the init program in initrd tarball!");
    let init_ctx = unsafe { VirtualMemoryManagerContext::create() };
    let entrypoint_vaddr = load_init_into_memory(&init_elf.file_data, elf, &init_ctx);
    let (initrd_vaddr, initrd_size) = load_initrd_into_memory(initrd, &init_ctx);
    let stack_top_vaddr = allocate_init_stack(&init_ctx);
    let arg = load_boot_info_into_memory(&init_ctx, initrd_vaddr, initrd_size);

    scheduler.spawn(TaskSpec::User {
        virtual_memory_manager_context: Arc::new(init_ctx),
        user_stack_vaddr: stack_top_vaddr,
        entrypoint_vaddr,
        arg,
    });
}
