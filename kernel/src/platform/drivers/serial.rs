mod bindings {
    include_bindings!("drivers/serial.rs");
}

pub struct SerialDriver;

impl SerialDriver {
    pub unsafe fn write(bytes: &[u8]) {
        unsafe {
            for byte in bytes {
                bindings::serial_write(*byte);
            }
        }
    }

    pub unsafe fn print(message: &str) {
        unsafe {
            Self::write(message.as_bytes());
        }
    }
}
