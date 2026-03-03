#if defined (__aarch64__)

#include "serial.h"

#include <stdint.h>
#include <stddef.h>
#include "syscalls.h"

// Private register offsets for the PL011
#define UART_DR    (0x00 / 4)
#define UART_FR    (0x18 / 4)
#define UART_IBRD  (0x24 / 4)
#define UART_FBRD  (0x28 / 4)
#define UART_LCRH  (0x2C / 4)
#define UART_CR    (0x30 / 4)
#define UART_IMSC  (0x38 / 4) // Interrupt Mask Set/Clear
#define UART_MIS   (0x40 / 4) // Masked Interrupt Status
#define UART_ICR   (0x44 / 4) // Interrupt Clear Register

// Register bits
#define FR_RXFE    (1 << 4) // Receive FIFO Empty
#define FR_TXFF    (1 << 5) // Transmit FIFO Full
#define INT_RX     (1 << 4) // Receive Interrupt bit

static volatile uint32_t *uart_base = NULL;

/**
 * serial_init: Should be called VERY early.
 * Only sets up basic MMIO mapping and Polling-based TX/RX.
 */
bool serial_init(void) {
    const uintptr_t physical_base = 0x9000000;
    uintptr_t virtual_base = 0x7FFFFF900000;
    sys_err_t err;
    if ((err = sys_mmap_dev(virtual_base, 0x1000, physical_base, 0, 0, &virtual_base))) {
        const char *err_msg = sys_err_get_message(err);
        size_t len;
        for (len = 0; err_msg[len] != '\0'; len++) {}
        sys_write(1, err_msg, len);
        return true;
    }

    uart_base = (volatile uint32_t *) virtual_base;

    // Hardware Reset - Disable everything first
    uart_base[UART_CR] = 0;
    uart_base[UART_ICR] = 0x7FF; // Clear all sticky interrupts

    // 8N1 + Enable FIFOs. We enable FIFOs here so hardware starts
    // buffering keys even before we enable interrupts.
    uart_base[UART_LCRH] = (3 << 5) | (1 << 4);

    // Enable UART, TX, and RX (Polling mode is now active)
    uart_base[UART_CR] = (1 << 0) | (1 << 8) | (1 << 9);

    return false;
}

static int is_transmit_empty() {
    if (!uart_base) return 0;
    return !(uart_base[UART_FR] & FR_TXFF);
}

static void write_serial(char a) {
    if (!uart_base) return;

    if (a == '\n') {
        write_serial('\r');
    }

    while (!is_transmit_empty()) {
        __asm__ volatile("yield");
    }

    uart_base[UART_DR] = (uint32_t) a;
}

void serial_write(const uint8_t byte) {
    if (!uart_base) return;

    if (byte == '\n') {
        write_serial('\r');
    }

    while (!is_transmit_empty()) {
        __asm__ volatile("yield");
    }

    uart_base[UART_DR] = (uint32_t) byte;
}

void serial_print(const char *message) {
    if (!message) return;
    for (size_t i = 0; message[i] != '\0'; i++) {
        write_serial(message[i]);
    }
}

void serial_println(const char *message) {
    serial_print(message);
    write_serial('\n');
}

void serial_print_hex_u64(uint64_t value) {
    static const char *hex = "0123456789abcdef";
    serial_print("0x");
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nib = (value >> i) & 0xF;
        write_serial(hex[nib]);
    }
}

#endif
