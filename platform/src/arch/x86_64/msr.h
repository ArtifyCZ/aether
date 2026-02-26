#pragma once

#include <stdint.h>

void msr_init(void);

void msr_set_kernel_stack(uint64_t stack);
