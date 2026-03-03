use crate::platform::drivers::serial::SerialDriver;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = writeln!($crate::logging::Logger, $($arg)*);
    });
}

pub struct Logger;

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            SerialDriver::print(s);
        }
        Ok(())
    }
}
