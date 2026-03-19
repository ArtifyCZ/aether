use core::ffi::c_void;
use core::ptr::NonNull;
use crate::arch::x86_64::gdt;
use crate::early_console;

#[unsafe(no_mangle)]
unsafe extern "C" fn platform_init(config: *const kernel_bindings_gen::platform_config) {
    unsafe {
        init(NonNull::new(config.read().rsdp_address));
    }
}

unsafe extern "C" {
    fn gdt_init();
    fn msr_init();
    fn acpi_init(rsdp_address: *mut c_void);
}

pub unsafe fn init(rsdp_address: Option<NonNull<c_void>>) {
    unsafe {
        early_console::print("Setting up GDT...");
        gdt::init();
        early_console::print("GDT initialized!");

        early_console::print("Setting up MSR...");
        msr_init();
        early_console::print("MSR initialized!");

        if let Some(rsdp_address) = rsdp_address {
            early_console::print("Initializing ACPI...");
            acpi_init(rsdp_address.as_ptr());
            early_console::print("ACPI initialized!");
        } else {
            early_console::print("No RSDP found!");
        }
    }
}
