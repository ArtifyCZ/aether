#pragma once

#include <stdbool.h>
#include <stdint.h>

bool serial_init(void);

void serial_print(const char *message);

void serial_println(const char *message);

void serial_print_hex_u64(uint64_t value);
