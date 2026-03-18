// NOTE THAT THE IMPLEMENTATION HAS BEEN MOVED TO RUST!

#pragma once

#include <stdint.h>

/*
 * NOTE THAT THIS CONSOLE IS MEANT TO BE USED ONLY UNTIL SWITCHING TO USERSPACE!!!
 * USING IT AFTERWARD RESULTS IN UNDEFINED BEHAVIOR AND POSSIBLY EVEN A KERNEL PANIC!!!
 */

void early_console_print(const char *message);

void early_console_println(const char *message);

void early_console_print_hex_u64(uint64_t value);
