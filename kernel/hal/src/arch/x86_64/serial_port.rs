use crate::arch::interface::serial_port::ArchSerialPort;
use core::arch::asm;

unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let ret: u8;
        asm!(
            "inb %dx, %al",
            out("al") ret,
            in("dx") port,
            options(att_syntax),
        );
        ret
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "outb %al, %dx",
            in("al") value,
            in("dx") port,
            options(att_syntax),
        );
    }
}

pub struct ArchSerialPortImpl {
    port: u16,
}

pub struct SerialPortConfig {
    pub port: u16,
}

impl ArchSerialPort for ArchSerialPortImpl {
    type InitConfig = SerialPortConfig;

    unsafe fn init(SerialPortConfig { port }: SerialPortConfig) -> Self {
        unsafe {
            outb(port + 1, 0x00); // Disable all interrupts
            outb(port + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(port + 1, 0x00); //                  (hi byte)
            outb(port + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(port + 4, 0x0B); // IRQs enabled, RTS/DSR set
            outb(port + 4, 0x1E); // Set in loopback mode, test the serial chip
            outb(port + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

            // Check if serial is faulty (i.e.: different byte from sent one)
            if inb(port + 0) != 0xAE {
                panic!("Serial faulty");
            }

            // If serial is not faulty, set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            outb(port + 4, 0x0F);

            Self { port }
        }
    }

    unsafe fn is_transmit_empty(&mut self) -> bool {
        unsafe { inb(self.port + 5) & 0x20 != 0 }
    }

    unsafe fn send_byte(&mut self, byte: u8) {
        unsafe {
            while !self.is_transmit_empty() {
                asm!("pause");
            }

            #[expect(clippy::identity_op)]
            outb(self.port + 0, byte);
        }
    }
}
