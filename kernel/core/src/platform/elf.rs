use crate::platform::virtual_memory_manager_context::VirtualMemoryManagerContext;

mod bindings {
    include_bindings!("elf.rs");
}

pub struct Elf;

impl Elf {
    pub unsafe fn init(hhdm_offset: u64) {
        unsafe {
            bindings::elf_init(hhdm_offset as usize);
        }
    }

    pub unsafe fn load(vmm_ctx: &VirtualMemoryManagerContext, data: &[u8]) -> Option<usize> {
        unsafe {
            let ctx = core::mem::transmute(vmm_ctx.inner());
            let data = core::mem::transmute(data.as_ptr());
            let mut entrypoint_vaddr: usize = 0;
            bindings::elf_load(ctx, data, &raw mut entrypoint_vaddr);
            Some(entrypoint_vaddr)
        }
    }
}
