use crate::arch;
use core::ffi::{c_char, CStr};

static mut SERIAL_BASE: u64 = 0;

pub unsafe fn init(serial_base: u64) {
    unsafe {
        SERIAL_BASE = arch::emergency_console::init(serial_base);
        print("\n\n");
        print("========================================\n");
        print("========    EMERGENCY CONSOLE   ========\n");
        print("========================================\n");
        print("\n");
    }
}

pub unsafe fn print(message: &str) {
    unsafe {
        if SERIAL_BASE == 0 {
            #[cfg(target_arch = "x86_64")]
            init(0x3f8);
            #[cfg(target_arch = "aarch64")]
            init(0x9000000);
        }

        for byte in message.bytes() {
            arch::emergency_console::write(SERIAL_BASE, byte);
        }
    }
}
