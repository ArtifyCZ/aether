use core::ffi::c_void;
use kernel_bindings_gen::{platform_config, platform_init};

pub struct Platform;

impl Platform {
    pub unsafe fn init(rsdp_address: u64) {
        unsafe {
            let config = platform_config {
                rsdp_address: rsdp_address as *mut c_void,
            };
            platform_init(&config);
        }
    }
}
