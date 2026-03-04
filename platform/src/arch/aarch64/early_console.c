#include "early_console.h"

#include "boot.h"
#include <stdint.h>

#include <stddef.h>
#include "interrupts.h"
#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

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
#define FR_BUSY    (1 << 3)
#define INT_RX     (1 << 4) // Receive Interrupt bit

static volatile uint32_t *uart_base = NULL;

void early_console_init(uintptr_t serial_base) {
    const uintptr_t virtual_base = vaa_alloc_range(VMM_PAGE_SIZE);
    uart_base = (volatile uint32_t *) virtual_base;

    if (!vmm_map_page(
        &g_kernel_context,
        virtual_base,
        serial_base,
        VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE
    )) {
        hcf();
    }

    // Hardware Reset - Disable everything first
    uart_base[UART_CR] = 0;
    uart_base[UART_ICR] = 0x7FF; // Clear all sticky interrupts

    // 8N1 + Enable FIFOs. We enable FIFOs here so hardware starts
    // buffering keys even before we enable interrupts.
    uart_base[UART_LCRH] = (3 << 5) | (1 << 4);

    // Enable UART, TX, and RX (Polling mode is now active)
    uart_base[UART_CR] = (1 << 0) | (1 << 8) | (1 << 9);
}

void early_console_disable(void) {
    if (!uart_base) return;

    // Wait for the UART to no longer be busy
    // This ensures the FIFO and the transmit shifter are both empty.
    while (uart_base[UART_FR] & FR_BUSY) {
        __asm__ volatile("yield");
    }

    // Disable UART, TX, and RX in the Control Register
    uart_base[UART_CR] = 0;

    // Mask all interrupts just in case
    uart_base[UART_IMSC] = 0;

    // Clear any remaining sticky interrupt flags
    uart_base[UART_ICR] = 0x7FF;

    // Invalidate the pointer so accidental calls return immediately
    // Note: In a microkernel, you might also want to unmap the page here
    // using vmm_unmap_page to be truly 'hard'.
    uart_base = NULL;
}

static int is_transmit_empty() {
    if (!uart_base)
        return 0;
    return !(uart_base[UART_FR] & FR_TXFF);
}

static void write_serial(char a) {
    if (!uart_base)
        return;

    if (a == '\n') {
        write_serial('\r');
    }

    while (!is_transmit_empty()) {
        __asm__ volatile("yield");
    }

    uart_base[UART_DR] = (uint32_t) a;
}

void early_console_write(const uint8_t byte) {
    if (!uart_base)
        return;

    if (byte == '\n') {
        write_serial('\r');
    }

    while (!is_transmit_empty()) {
        __asm__ volatile("yield");
    }

    uart_base[UART_DR] = (uint32_t) byte;
}

void early_console_print(const char *message) {
    if (!message)
        return;
    for (size_t i = 0; message[i] != '\0'; i++) {
        write_serial(message[i]);
    }
}

void early_console_println(const char *message) {
    early_console_print(message);
    write_serial('\n');
}

void early_console_print_hex_u64(const uint64_t value) {
    static const char *hex = "0123456789abcdef";
    early_console_print("0x");
    for (int i = 60; i >= 0; i -= 4) {
        uint8_t nib = (value >> i) & 0xF;
        write_serial(hex[nib]);
    }
}
