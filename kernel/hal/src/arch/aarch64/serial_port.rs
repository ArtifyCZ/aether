use crate::arch::interface::serial_port::ArchSerialPort;
use atom_primitives::HhdmOffset;

const UART_DATA: usize = 0x00;
const UART_FLAG: usize = 0x18;
const UART_FLAG_TX_FULL: u32 = 1 << 5;

pub struct ArchSerialPortImpl {
    base: usize,
}

pub struct SerialPortConfig {
    pub uart_phys_base: usize,
    pub hhdm_offset: HhdmOffset,
}

impl ArchSerialPort for ArchSerialPortImpl {
    type InitConfig = SerialPortConfig;

    unsafe fn init(
        SerialPortConfig {
            uart_phys_base,
            hhdm_offset,
        }: SerialPortConfig,
    ) -> Self {
        Self {
            base: uart_phys_base + *hhdm_offset,
        }
    }

    unsafe fn is_transmit_empty(&mut self) -> bool {
        let flags = unsafe { ((self.base + UART_FLAG) as *const u32).read_volatile() };
        flags & UART_FLAG_TX_FULL == 0
    }

    unsafe fn send_byte(&mut self, byte: u8) {
        while !unsafe { self.is_transmit_empty() } {
            core::hint::spin_loop();
        }
        unsafe {
            ((self.base + UART_DATA) as *mut u32).write_volatile(byte as u32);
        }
    }
}
