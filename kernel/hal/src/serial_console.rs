use crate::arch::current::serial_port::ArchSerialPortImpl;
use crate::arch::interface::serial_port::ArchSerialPort;
use core::fmt::Write;

pub struct SerialConsole {
    serial: ArchSerialPortImpl,
}

unsafe impl Send for SerialConsole {}

pub type SerialPortConfig = <ArchSerialPortImpl as ArchSerialPort>::InitConfig;

/// @TODO: use token pattern to ensure initialization just once
///
/// # Safety
///
/// Should not be called more than once.
pub unsafe fn init(config: SerialPortConfig) -> SerialConsole {
    unsafe {
        SerialConsole {
            serial: ArchSerialPortImpl::init(config),
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
