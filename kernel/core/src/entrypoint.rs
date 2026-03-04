use crate::main;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(hhdm_offset: u64, rsdp_address: u64) {
    main(hhdm_offset, rsdp_address);
}
