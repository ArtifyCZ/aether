#pragma once

#include <stdint.h>

#define KERNEL_CODE_SEGMENT 0x08
#define KERNEL_DATA_SEGMENT 0x10
#define USER_DATA_SEGMENT 0x20
#define USER_CODE_SEGMENT 0x28

void gdt_set_kernel_stack(uintptr_t stack);
