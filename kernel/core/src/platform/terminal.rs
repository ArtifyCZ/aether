use core::ffi::c_char;

mod bindings {
    include_bindings!("terminal.rs");
}

pub struct Terminal;

impl Terminal {
    pub unsafe fn print_char(c: char) {
        unsafe {
            Self::print_byte(c as c_char);
        }
    }

    pub unsafe fn print_byte(c: c_char) {
        unsafe {
            bindings::terminal_print_char(c);
        }
    }

    pub unsafe fn print_bytes(bytes: &[u8]) {
        for byte in bytes {
            unsafe {
                Self::print_byte(*byte as c_char);
            }
        }
    }

    pub unsafe fn print(message: &str) {
        for c in message.chars() {
            unsafe {
                Self::print_char(c);
            }
        }
    }
}
