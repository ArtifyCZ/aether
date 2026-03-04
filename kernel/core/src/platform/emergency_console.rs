mod bindings {
    include_bindings!("emergency_console.rs");
}

pub struct EmergencyConsole;

impl EmergencyConsole {
    pub unsafe fn init() {
        let serial_base;

        #[cfg(target_arch = "x86_64")]
        {
            serial_base = 0x3f8;
        }

        #[cfg(target_arch = "aarch64")]
        {
            serial_base = 0x9000000;
        }

        unsafe {
            bindings::emergency_console_init(serial_base);
        }
    }

    pub unsafe fn write(byte: u8) {
        unsafe {
            bindings::emergency_console_write(byte);
        }
    }

    pub unsafe fn write_str(s: &str) {
        unsafe {
            for byte in s.bytes() {
                Self::write(byte);
            }
        }
    }
}
