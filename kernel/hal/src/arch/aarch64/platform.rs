#[unsafe(no_mangle)]
unsafe extern "C" fn platform_init(config: *const kernel_bindings_gen::platform_config) {
    unsafe {
        let _ = config;
        init();
    }
}

unsafe extern "C" {
    fn cpu_local_init();
}

pub unsafe fn init() {
    unsafe {
        cpu_local_init();
    }
}
