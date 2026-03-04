use core::ffi::c_void;

mod bindings {
    include_bindings!("platform.rs");
}

pub struct Platform;

impl Platform {
    pub unsafe fn init(rsdp_address: u64) {
        unsafe {
            let config = bindings::platform_config {
                rsdp_address: rsdp_address as *mut c_void,
            };
            bindings::platform_init(&config);
        }
    }
}
