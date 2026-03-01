#pragma once

#if defined (__x86_64__)

#include <stdbool.h>
#include <stdint.h>

bool keyboard_arch_parse_scancode(uint8_t sc, char *out);

#endif
