use core::ffi::c_void;
use core::ptr::NonNull;
use crate::arch::x86_64::{acpi, gdt, msr};
use crate::early_console;

#[unsafe(no_mangle)]
unsafe extern "C" fn platform_init(config: *const kernel_bindings_gen::platform_config) {
    unsafe {
        init(NonNull::new(config.read().rsdp_address));
    }
}

pub unsafe fn init(rsdp_address: Option<NonNull<c_void>>) {
    unsafe {
        early_console::print("Setting up GDT...\n");
        gdt::init();
        early_console::print("GDT initialized!\n");

        early_console::print("Setting up MSR...\n");
        msr::init();
        early_console::print("MSR initialized!\n");

        if let Some(rsdp_address) = rsdp_address {
            early_console::print("Initializing ACPI...\n");
            acpi::init(rsdp_address.as_ptr() as usize);
            early_console::print("ACPI initialized!\n");
        } else {
            early_console::print("No RSDP found!\n");
        }
    }
}
