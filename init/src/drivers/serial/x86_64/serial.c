#include "drivers/serial/serial.h"

#include <stdint.h>
#include <stddef.h>

#include "../../../io.x86_64.h"

#define PORT 0x3f8

/**
 * @TODO: replace PORT constant with the base parameter
 */
bool serial_init(void) {
    outb(PORT + 1, 0x00); // Disable all interrupts
    outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
    outb(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    outb(PORT + 1, 0x00); //                  (hi byte)
    outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
    outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
    outb(PORT + 0, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

    // Check if serial is faulty (i.e: not same byte as sent)
    if (inb(PORT + 0) != 0xAE) {
        return 1;
    }

    // If serial is not faulty set it in normal operation mode
    // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
    outb(PORT + 4, 0x0F);
    return 0;
}

static int is_transmit_empty() {
    return inb(PORT + 5) & 0x20;
}

static void write_serial(char a) {
    while (is_transmit_empty() == 0) {
        __asm__ volatile ("pause");
    }

    outb(PORT, a);
}

void serial_write(const uint8_t byte) {
    while (is_transmit_empty() == 0) {
        __asm__ volatile ("pause");
    }

    outb(PORT, byte);
}

void serial_print(const char *message) {
    if (message != NULL) {
        for (size_t i = 0; message[i] != 0x00; i++) {
            write_serial(message[i]);
        }
    }
}

void serial_println(const char *message) {
    if (message != NULL) {
        for (size_t i = 0; message[i] != 0x00; i++) {
            write_serial(message[i]);
        }
    }
    write_serial('\n');
}

void serial_print_hex_u64(uint64_t value) {
    static const char *hex = "0123456789abcdef";
    serial_print("0x");
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nib = (value >> (uint64_t) i) & 0xF;
        write_serial(hex[nib]);
    }
}
