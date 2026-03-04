#pragma once

#include <stdint.h>

/*
 * NOTE THAT THIS SHALL BE USED ONLY IN EMERGENCY CASES WHEN THERE IS NO RECOVERY!!!
 * DO NOT EVER TRY TO RECOVER WHEN EMERGENCY CONSOLE HAS BEEN USED!!!
 */

void emergency_console_init(uintptr_t serial_base);

void emergency_console_write(uint8_t byte);

void emergency_console_print(const char *message);

void emergency_console_println(const char *message);

void emergency_console_print_hex_u64(uint64_t value);
