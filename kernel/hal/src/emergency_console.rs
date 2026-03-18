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

#[unsafe(no_mangle)]
unsafe extern "C" fn emergency_console_print(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_bytes();
        for byte in message {
            arch::emergency_console::write(SERIAL_BASE, *byte);
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn emergency_console_println(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_bytes();
        for byte in message {
            arch::emergency_console::write(SERIAL_BASE, *byte);
        }
        arch::emergency_console::write(SERIAL_BASE, '\n' as u8);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn emergency_console_print_hex_u64(value: u64) {
    unsafe {
        static HEX: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
        print("0x");
        let mut i = 60;
        while i >= 0 {
            let nib = ((value >> i) & 0xF) as u8;
            arch::emergency_console::write(SERIAL_BASE, HEX[nib as usize] as u8);
            i -= 4;
        }
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
