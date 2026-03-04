mod bindings {
    include_bindings!("early_console.rs");
}

pub struct EarlyConsole;

impl EarlyConsole {
    pub unsafe fn disable() {
        unsafe {
            bindings::early_console_disable();
        }
    }

    pub unsafe fn write(byte: u8) {
        unsafe {
            bindings::early_console_write(byte);
        }
    }

    pub unsafe fn write_str(str: &str) {
        for byte in str.bytes() {
            unsafe {
                EarlyConsole::write(byte);
            }
        }
    }
}
