#pragma once

#include <stdint.h>

// Halt and catch fire function.
_Noreturn void hcf(void);

// The following will be our kernel's entry point.
// If renaming boot() to something else, make sure to change the
// linker script accordingly.
__attribute__((used)) void boot(void);

extern void kernel_main(uint64_t hhdm_offset, uintptr_t rsdp_address);
