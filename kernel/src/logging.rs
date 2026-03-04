use core::sync::atomic::{AtomicBool, Ordering};
use crate::platform::early_console::EarlyConsole;
use crate::platform::emergency_console::EmergencyConsole;
use crate::platform::terminal::Terminal;

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = writeln!($crate::logging::Logger, $($arg)*);
    });
}

static USE_EMERGENCY_CONSOLE: AtomicBool = AtomicBool::new(false);

pub struct Logger;

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            if USE_EMERGENCY_CONSOLE.load(Ordering::Acquire) {
                EmergencyConsole::write_str(s);
                return Ok(());
            }

            EarlyConsole::write_str(s);
            Terminal::print(s);
        }
        Ok(())
    }
}

pub unsafe fn switch_to_emergency_console() {
    USE_EMERGENCY_CONSOLE.store(true, Ordering::SeqCst);
    unsafe {
        EmergencyConsole::init();
    }
}
