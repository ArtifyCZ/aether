use crate::platform::drivers::serial::SerialDriver;
use crate::platform::terminal::Terminal;

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
            Terminal::print(s);
        }
        Ok(())
    }
}
