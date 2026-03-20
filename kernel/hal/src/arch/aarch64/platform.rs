use crate::early_console;

#[unsafe(no_mangle)]
unsafe extern "C" fn platform_init(config: *const kernel_bindings_gen::platform_config) {
    unsafe {
        let _ = config;
        init();
    }
}

pub unsafe fn init() {
    unsafe {
        early_console::print("Initializing CPU Local Storage...\n");
        crate::arch::aarch64::cpu_local::init();
        early_console::print("CPU Local Storage initialized!\n");
    }
}
