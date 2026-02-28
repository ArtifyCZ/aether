use core::str::FromStr;

use alloc::{ffi::CString, sync::Arc};

use crate::{
    interrupt_safe_spin_lock::InterruptSafeSpinLock,
    platform::{
        elf::Elf, modules::Modules, tasks::TaskContext,
        virtual_memory_manager_context::VirtualMemoryManagerContext,
    },
};
use crate::platform::memory_layout::PAGE_FRAME_SIZE;
use crate::platform::physical_memory_manager::PhysicalMemoryManager;
use crate::platform::virtual_memory_manager_context::VirtualMemoryMappingFlags;
use crate::platform::virtual_page_address::VirtualPageAddress;
use crate::scheduler::Scheduler;

fn load_init_into_memory(init_ctx: &VirtualMemoryManagerContext) -> usize {
    let init_elf_string = CString::from_str("init.elf").expect("Failed to create CString");
    let init_elf = unsafe { Modules::find(init_elf_string.as_c_str()) }.unwrap();
    let entrypoint_vaddr = unsafe { Elf::load(init_ctx, init_elf) }.unwrap();
    entrypoint_vaddr
}

fn allocate_init_stack(init_ctx: &VirtualMemoryManagerContext) -> usize {
    const INIT_STACK_TOP_VADDR: usize = 0x7FFFFFFFF000usize;
    for i in 0..4 {
        // allocate 4 pages as stack
        let page_vaddr = INIT_STACK_TOP_VADDR - (i + 1) * PAGE_FRAME_SIZE;
        #[allow(mutable_transmutes)]
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

pub fn spawn_init_process(scheduler: &Scheduler) {
    let init_ctx = unsafe { VirtualMemoryManagerContext::create() };
    let entrypoint_vaddr = load_init_into_memory(&init_ctx);
    let stack_top_vaddr = allocate_init_stack(&init_ctx);

    let task = TaskContext::new_user(Arc::new(init_ctx), stack_top_vaddr, entrypoint_vaddr);
    scheduler.add(task);
}
