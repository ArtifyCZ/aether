use crate::arch::cpu::hcf;
use crate::mmu;
use core::arch::asm;
use mmu::VirtualMemoryMappingFlags;

const UART_DR: usize = 0x00;
const UART_FR: usize = 0x06;
const UART_IBRD: usize = 0x09;
const UART_FBRD: usize = 0x10;
const UART_LCRH: usize = 0x2C / 4;
const UART_CR: usize = 0x30 / 4;
const UART_IMSC: usize = 0x38 / 4; // Interrupt Mask Set/Clear
const UART_MIS: usize = 0x40 / 4; // Masked Interrupt Status
const UART_ICR: usize = 0x44 / 4; // Interrupt Clear Register

// Register bits
const FR_RXFE: u32 = 1 << 4; // Receive FIFO Empty
const FR_TXFF: u32 = 1 << 5; // Transmit FIFO Full
const FR_BUSY: u32 = 1 << 3;
const INT_RX: u32 = 1 << 4; // Receive Interrupt bit

pub unsafe fn init(serial_base: u64) -> u64 {
    unsafe {
        asm!("msr daifset, #3", "dmb sy");

        let virtual_base =
            kernel_bindings_gen::vaa_alloc_range(kernel_bindings_gen::VMM_PAGE_SIZE as usize);

        let kernel_context = mmu::get_kernel_context();
        if !mmu::map_page(
            kernel_context,
            virtual_base,
            serial_base as usize,
            VirtualMemoryMappingFlags::PRESENT
                | VirtualMemoryMappingFlags::WRITE
                | VirtualMemoryMappingFlags::DEVICE,
        ) {
            hcf();
        }

        let uart_base = virtual_base as *mut u32;

        // Hardware Reset - Disable everything first
        uart_base.add(UART_CR).write_volatile(0);
        uart_base.add(UART_ICR).write_volatile(0x7FF); // Clear all sticky interrupts

        // 8N1 + Enable FIFOs. We enable FIFOs here so hardware starts
        // buffering keys even before we enable interrupts.
        uart_base.add(UART_LCRH).write_volatile((3 << 5) | (1 << 4));

        // Enable UART, TX, and RX (Polling mode is now active)
        uart_base
            .add(UART_CR)
            .write_volatile((1 << 0) | (1 << 8) | (1 << 9));

        uart_base as u64
    }
}

pub unsafe fn disable(serial_base: u64) {
    unsafe {
        let uart_base = serial_base as *mut u32;

        // Wait for the UART to no longer be busy
        // This ensures the FIFO and the transmit shifter are both empty.
        while (uart_base.add(UART_FR).read_volatile() & FR_BUSY) != 0 {
            asm!("yield");
        }

        // Disable UART, TX, and RX in the Control Register
        uart_base.add(UART_CR).write_volatile(0);

        // Mask all interrupts just in case
        uart_base.add(UART_IMSC).write_volatile(0);

        // Clear any remaining sticky interrupt flags
        uart_base.add(UART_ICR).write_volatile(0x7FF);
    }
}

unsafe fn is_transmit_empty(uart_base: *mut u32) -> bool {
    (uart_base.add(UART_FR).read_volatile() & FR_TXFF) == 0
}

pub unsafe fn write(serial_base: u64, byte: u8) {
    unsafe {
        let uart_base = serial_base as *mut u32;

        if byte == ('\n' as u8) {
            write(serial_base, '\r' as u8);
        }

        while !is_transmit_empty(uart_base) {
            asm!("yield");
        }

        uart_base.add(UART_DR).write_volatile(byte as u32);
    }
}
