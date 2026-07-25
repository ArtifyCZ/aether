use kernel_hal::emergency_console;

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
            emergency_console::init(serial_base);
        }
    }

    pub unsafe fn write_str(s: &str) {
        unsafe {
            emergency_console::print(s);
        }
    }
}
