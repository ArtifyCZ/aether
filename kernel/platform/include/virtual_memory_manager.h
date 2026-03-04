#pragma once

#include <stdint.h>
#include <stdbool.h>

#define VMM_PAGE_SIZE 0x1000ull

// Generic flags that work across architectures
typedef enum {
    VMM_FLAG_PRESENT = 1 << 0,
    VMM_FLAG_WRITE   = 1 << 1,
    VMM_FLAG_USER    = 1 << 2,
    VMM_FLAG_EXEC    = 1 << 3,
    VMM_FLAG_DEVICE  = 1 << 4, // Critical: maps to PCD/PWT on x86, MAIR on AArch64
    VMM_FLAG_NOCACHE = 1 << 5,
} vmm_flags_t;

struct vmm_context {
    uintptr_t root; // Physical address of L0 (AArch64) or L4 (x86_64)
};

extern struct vmm_context g_kernel_context;

void vmm_init(uint64_t hhdm_offset);

struct vmm_context vmm_context_create(void);

bool vmm_map_page(const struct vmm_context *context, uintptr_t virt, uintptr_t phys, vmm_flags_t flags);

bool vmm_unmap_page(const struct vmm_context *context, uintptr_t virt);

uintptr_t vmm_translate(const struct vmm_context *context, uintptr_t virt);
