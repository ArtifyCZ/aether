use atom_hal::sync::Spinlock;
use core::fmt::Write;
use log::{LevelFilter, Log, Metadata, Record};

struct KernelLoggerInner {
    early_console: &'static mut dyn EarlyConsole,
}

static INNER: Spinlock<Option<KernelLoggerInner>> = Spinlock::new(None);

struct KernelLogger;

static INSTANCE: KernelLogger = KernelLogger;

pub trait EarlyConsole: Write + Send {
    /// This should disable the early console.
    /// Once the early console has been disabled,
    /// any attempt to write through it should
    /// result in a panic.
    fn close(&mut self);
}

pub fn init(early_console: &'static mut dyn EarlyConsole) {
    let mut inner = INNER.lock();
    assert!(inner.is_none(), "Cannot initialize logger twice");
    inner.replace(KernelLoggerInner { early_console });
    log::set_logger(&INSTANCE)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
}

impl Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LevelFilter::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut inner = INNER.lock();
            let _ = writeln!(
                inner.as_mut().unwrap().early_console,
                "[{}] {}",
                record.level(),
                record.args(),
            );
        }
    }

    fn flush(&self) {}
}
