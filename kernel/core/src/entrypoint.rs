use crate::main;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(
    hhdm_offset: u64,
    memmap: *mut kernel_bindings_gen::limine_memmap_response,
    framebuffer: *mut kernel_bindings_gen::limine_framebuffer,
    modules: *mut kernel_bindings_gen::limine_module_response,
    rsdp_address: u64,
) -> ! {
    main(hhdm_offset, memmap, framebuffer, modules, rsdp_address);
}
