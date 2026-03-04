#pragma once

#include <stdint.h>

/*
 * NOTE THAT THIS CONSOLE IS MEANT TO BE USED ONLY UNTIL SWITCHING TO USERSPACE!!!
 * USING IT AFTERWARD RESULTS IN UNDEFINED BEHAVIOR AND POSSIBLY EVEN A KERNEL PANIC!!!
 */

void early_console_init(uintptr_t serial_base);

void early_console_disable(void);

void early_console_write(uint8_t byte);

void early_console_print(const char *message);

void early_console_println(const char *message);

void early_console_print_hex_u64(uint64_t value);
