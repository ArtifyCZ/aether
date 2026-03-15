#pragma once

#include <stdint.h>
#include "limine.h"

// Halt and catch fire function.
_Noreturn void hcf(void);

// The following will be our kernel's entry point.
// If renaming boot() to something else, make sure to change the
// linker script accordingly.
__attribute__((used)) void boot(void);

__attribute__((noreturn))
extern void kernel_main(
    uint64_t hhdm_offset,
    struct limine_memmap_response *memmap,
    struct limine_framebuffer *framebuffer,
    struct limine_module_response *modules,
    uintptr_t rsdp_address
);
