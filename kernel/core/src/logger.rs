use atom_hal::framebuffer_console::{Framebuffer, FramebufferConsole};
use atom_hal::serial_console::{SerialConsole, SerialConsoleConfig};
use atom_hal::sync::Spinlock;
use atom_hal::{framebuffer_console, serial_console};
use core::fmt::Write;
use log::{Level, LevelFilter, Log, Metadata, Record};

struct KernelLoggerInner {
    serial_console: Option<SerialConsole>,
    framebuffer_console: Option<FramebufferConsole>,
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

pub fn init(framebuffer: Option<Framebuffer>, serial_console_config: Option<SerialConsoleConfig>) {
    let mut inner = INNER.lock();
    assert!(inner.is_none(), "Cannot initialize logger twice");
    let framebuffer_console = framebuffer.map(framebuffer_console::init);
    let serial_console = serial_console_config.map(serial_console::init);
    inner.replace(KernelLoggerInner {
        framebuffer_console,
        serial_console,
    });
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
                inner.as_mut().unwrap(),
                "[{}] {}",
                match record.level() {
                    Level::Error => "\x1b[1;31mERROR\x1b[0m",
                    Level::Warn => "\x1b[1;33mWARN\x1b[0m",
                    Level::Info => "\x1b[1;32mINFO\x1b[0m",
                    Level::Debug => "\x1b[1;32mDEBUG\x1b[0m",
                    Level::Trace => "\x1b[1;32mTRACE\x1b[0m",
                },
                record.args(),
            );
        }
    }

    fn flush(&self) {}
}

impl Write for KernelLoggerInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(framebuffer_console) = &mut self.framebuffer_console {
            framebuffer_console.write_str(s)?;
        }
        if let Some(serial_console) = &mut self.serial_console {
            serial_console.write_str(s)?;
        }
        Ok(())
    }
}
