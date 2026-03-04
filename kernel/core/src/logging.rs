use crate::platform::early_console::EarlyConsole;
use crate::platform::emergency_console::EmergencyConsole;
use crate::platform::terminal::Terminal;
use core::sync::atomic::{AtomicBool, Ordering};

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = writeln!($crate::logging::Logger, $($arg)*);
    });
}

static USE_EARLY_CONSOLE: AtomicBool = AtomicBool::new(true);
static USE_EMERGENCY_CONSOLE: AtomicBool = AtomicBool::new(false);

pub struct Logger;

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            if USE_EMERGENCY_CONSOLE.load(Ordering::Acquire) {
                EmergencyConsole::write_str(s);
                return Ok(());
            }

            if USE_EARLY_CONSOLE.load(Ordering::Acquire) {
                EarlyConsole::write_str(s);
            }

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

pub unsafe fn disable_early_console() {
    USE_EARLY_CONSOLE.store(false, Ordering::SeqCst);
    unsafe {
        EarlyConsole::disable();
    }
}
