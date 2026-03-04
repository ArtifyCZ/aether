#include "virtual_address_allocator.h"

#include <stdint.h>
#include <stddef.h>

#include "virtual_memory_manager.h"

static volatile uintptr_t g_next_range_start = 0x0;

static inline uint64_t align_up_u64(uint64_t v, uint64_t a) {
    return (v + (a - 1)) & ~(a - 1);
}

void vaa_init(void) {
    g_next_range_start = 0xFFFFC00000000000ULL; // beginning address for allocating ranges
}

uintptr_t vaa_alloc_range(size_t size) {
    // align size to a page
    size = align_up_u64(size, VMM_PAGE_SIZE);
    uintptr_t ret = g_next_range_start;
    g_next_range_start += size + VMM_PAGE_SIZE; // empty page to catch, for example, stack overflow into other data
    return ret;
}
