use crate::arch;
use bitflags::{bitflags};
use kernel_bindings_gen::{
    vmm_flags_t_VMM_FLAG_DEVICE, vmm_flags_t_VMM_FLAG_EXEC, vmm_flags_t_VMM_FLAG_NOCACHE,
    vmm_flags_t_VMM_FLAG_PRESENT, vmm_flags_t_VMM_FLAG_USER, vmm_flags_t_VMM_FLAG_WRITE,
};

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct VirtualMemoryMappingFlags: u32 {
        const PRESENT = vmm_flags_t_VMM_FLAG_PRESENT;
        const WRITE = vmm_flags_t_VMM_FLAG_WRITE;
        const USER = vmm_flags_t_VMM_FLAG_USER;
        const EXEC = vmm_flags_t_VMM_FLAG_EXEC;
        const DEVICE = vmm_flags_t_VMM_FLAG_DEVICE;
        const NO_CACHE = vmm_flags_t_VMM_FLAG_NOCACHE;
    }
}

pub unsafe fn init(hhdm_offset: usize) {
    unsafe {
        arch::mmu::init(hhdm_offset);
    }
}

pub unsafe fn get_kernel_context() -> usize {
    unsafe {
        arch::mmu::get_kernel_context()
    }
}

pub unsafe fn create_context() -> usize {
    unsafe { arch::mmu::create_context() }
}

// bool vmm_map_page(const struct vmm_context *context, uintptr_t virt, uintptr_t phys, vmm_flags_t flags)
#[unsafe(no_mangle)]
unsafe extern "C" fn vmm_map_page(
    context: *const kernel_bindings_gen::vmm_context,
    virt: usize,
    phys: usize,
    flags: kernel_bindings_gen::vmm_flags_t,
) -> bool {
    unsafe {
        map_page(
            context.read().root,
            virt,
            phys,
            VirtualMemoryMappingFlags::from_bits_retain(flags),
        )
    }
}

pub unsafe fn map_page(
    table: usize,
    virt: usize,
    phys: usize,
    flags: VirtualMemoryMappingFlags,
) -> bool {
    unsafe { arch::mmu::map_page(table, virt, phys, flags) }
}

pub unsafe fn unmap_page(table: usize, virt: usize) -> bool {
    unsafe { arch::mmu::unmap_page(table, virt) }
}

pub unsafe fn translate(table: usize, virt: usize) -> usize {
    unsafe { arch::mmu::translate(table, virt) }
}
