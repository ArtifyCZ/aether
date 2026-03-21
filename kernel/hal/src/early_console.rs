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

pub unsafe fn print(message: &str) {
    unsafe {
        for byte in message.bytes() {
            arch::early_console::write(SERIAL_BASE, byte);
        }
    }
}
