use crate::arch;
use core::ffi::{c_char, CStr};

static mut SERIAL_BASE: u64 = 0;

pub unsafe fn init(serial_base: u64) {
    unsafe {
        SERIAL_BASE = arch::early_console::init(serial_base);
    }
}

pub unsafe fn disable() {
    unsafe {
        arch::early_console::disable(SERIAL_BASE);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn early_console_print(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_bytes();
        for byte in message {
            arch::early_console::write(SERIAL_BASE, *byte);
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn early_console_println(message: *const c_char) {
    unsafe {
        let message = CStr::from_ptr(message);
        let message = message.to_bytes();
        for byte in message {
            arch::early_console::write(SERIAL_BASE, *byte);
        }
        arch::early_console::write(SERIAL_BASE, '\n' as u8);
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn early_console_print_hex_u64(value: u64) {
    unsafe {
        static HEX: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
        print("0x");
        let mut i = 60;
        while i >= 0 {
            let nib = ((value >> i) & 0xF) as u8;
            arch::early_console::write(SERIAL_BASE, HEX[nib as usize] as u8);
            i -= 4;
        }
    }
}

pub unsafe fn print(message: &str) {
    unsafe {
        for byte in message.bytes() {
            arch::early_console::write(SERIAL_BASE, byte);
        }
    }
}
