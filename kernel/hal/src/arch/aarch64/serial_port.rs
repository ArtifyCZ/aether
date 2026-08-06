use crate::arch::interface::serial_port::ArchSerialPort;
use atom_primitives::HhdmOffset;

pub struct ArchSerialPortImpl;

pub struct SerialPortConfig {
    pub uart_phys_base: usize,
    pub hhdm_offset: HhdmOffset,
}

impl ArchSerialPort for ArchSerialPortImpl {
    type InitConfig = SerialPortConfig;

    unsafe fn init(SerialPortConfig { .. }: SerialPortConfig) -> Self {
        // @TODO: implement
        Self
    }

    unsafe fn is_transmit_empty(&mut self) -> bool {
        // @TODO: implement
        true
    }

    unsafe fn send_byte(&mut self, byte: u8) {
        let _ = byte;
        // @TODO: implement
    }
}
