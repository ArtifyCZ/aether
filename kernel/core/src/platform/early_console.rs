mod bindings {
    include_bindings!("early_console.rs");
}

pub struct EarlyConsole;

impl EarlyConsole {
    pub unsafe fn init(serial_base: usize) {
        unsafe {
            kernel_bindings_gen::early_console_init(serial_base);
        }
    }

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
