use crate::arch::current::serial_port::ArchSerialPortImpl;
use crate::arch::interface::serial_port::ArchSerialPort;
use core::fmt::Write;
use core::marker::PhantomData;

pub struct SerialConsole {
    serial: ArchSerialPortImpl,
}

unsafe impl Send for SerialConsole {}

pub struct SerialConsoleConfig {
    pub port_config: SerialPortConfig,
    pub unsafe_token: SerialConsoleUnsafeToken,
}

pub struct SerialConsoleUnsafeToken {
    _phantom: PhantomData<()>,
}

/// The [SerialConsoleUnsafeToken] and this constructor function exist
/// to make it unsafe to construct [SerialConsoleConfig].
///
/// The only use for this function is when creating [SerialConsoleConfig].
///
/// # Safety
///
/// The caller has to ensure the validity of the values passed into the fields of [SerialConsoleConfig].
///
/// This function is not unsafe on its own,
/// but it is the only way to construct [SerialConsoleUnsafeToken]
/// that is required to construct [SerialConsoleConfig].
/// The point of the token is to ensure constructing [SerialConsoleConfig] is marked as unsafe.
pub unsafe fn create_serial_console_unsafe_token() -> SerialConsoleUnsafeToken {
    SerialConsoleUnsafeToken {
        _phantom: PhantomData,
    }
}

pub type SerialPortConfig = <ArchSerialPortImpl as ArchSerialPort>::InitConfig;

pub fn init(config: SerialConsoleConfig) -> SerialConsole {
    unsafe {
        SerialConsole {
            serial: ArchSerialPortImpl::init(config.port_config),
        }
    }
}

impl Write for SerialConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                unsafe {
                    self.serial.send_byte(b'\r');
                }
            }

            unsafe {
                self.serial.send_byte(byte);
            }
        }
        Ok(())
    }
}

impl Drop for SerialConsole {
    fn drop(&mut self) {
        todo!("Implement drop as closing the serial")
    }
}
