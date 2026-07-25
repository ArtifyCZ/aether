use kernel_hal::early_console;

pub struct EarlyConsole;

impl EarlyConsole {
    pub unsafe fn init(serial_base: usize) {
        unsafe {
            early_console::init(serial_base as u64);
        }
    }

    pub unsafe fn disable() {
        unsafe {
            early_console::disable();
        }
    }

    pub unsafe fn write_str(str: &str) {
        unsafe {
            early_console::print(str);
        }
    }
}
