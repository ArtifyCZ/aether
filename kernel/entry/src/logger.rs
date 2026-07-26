use crate::early_console;
use core::fmt::Write;
use log::{LevelFilter, Log, Metadata, Record};

struct KernelLogger;

static KERNEL_LOGGER: KernelLogger = KernelLogger;

pub fn init() {
    log::set_logger(&KernelLogger)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
}

impl Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LevelFilter::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut early_console_logger = EarlyConsoleLogger;
            let _ = write!(
                &mut early_console_logger,
                "[{}] {}\n",
                record.level(),
                record.args(),
            );
        }
    }

    fn flush(&self) {}
}

struct EarlyConsoleLogger;

impl Write for EarlyConsoleLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        early_console::write_str(s);
        Ok(())
    }
}
