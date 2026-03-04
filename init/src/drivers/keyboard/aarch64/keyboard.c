#include "drivers/keyboard/keyboard.h"

#include "../../../syscalls.h"

#define UART_IRQ 0x01

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

static volatile uint32_t *g_uart_base = NULL;

static __attribute__((noreturn)) void keyboard_thread(void);

void print(const char *message);

void keyboard_init(void) {
    uintptr_t uart_base = 0x7FFFFFA00000;
    const uintptr_t uart_size = 0x1000;
    const uintptr_t uart_phys = 0x9000000;
    sys_err_t err;
    if ((err = sys_mmap_dev(uart_base, uart_size, uart_phys, 0, 0, &uart_base))) {
        print("FAILED TO MAP UART: ");
        print(sys_err_get_message(err));
        sys_exit();
    }
    g_uart_base = (volatile uint32_t *) uart_base;
    const uintptr_t stack_size = 0x4000;
    uintptr_t stack_base;
    if (sys_mmap(0x7FFFFFF54000, stack_size, 0, 0, &stack_base)) {
        print("FAILED TO ALLOCATE STACK!");
        sys_exit();
    }
    const uintptr_t stack_top = stack_base + stack_size;
    if (sys_clone(0, (void *) stack_top, keyboard_thread, NULL)) {
        print("FAILED TO CLONE THREAD!");
        sys_exit();
    }

    // Enable Receive Interrupt Mask in UART hardware
    g_uart_base[UART_IMSC] = INT_RX;
}

static __attribute__((noreturn)) void keyboard_thread(void) {
    while (1) {
        if (sys_irq_unmask(UART_IRQ)) {
            print("FAILED TO UNMASK KEYBOARD IRQ!");
            sys_exit();
        }

        if (sys_irq_wait(UART_IRQ)) {
            print("FAILED TO WAIT FOR KEYBOARD IRQ!");
            sys_exit();
        }

        if (g_uart_base[UART_MIS] & INT_RX) {
            // Drain the FIFO
            while (!(g_uart_base[UART_FR] & FR_RXFE)) {
                uint32_t data = g_uart_base[UART_DR];
                uint8_t ch = (uint8_t) (data & 0xFF);
                const char c = ch == '\r' ? '\n' : ch;
                const char message[] = {c, '\0'};
                print(message);
            }

            // Clear the interrupt
            g_uart_base[UART_ICR] = INT_RX;
        }
    }
}
