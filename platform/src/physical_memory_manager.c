#include "physical_memory_manager.h"

#include "limine.h"
#include "early_console.h"
#include "emergency_console.h"

static inline uint64_t align_up_u64(uint64_t v, uint64_t a) {
    return (v + (a - 1)) & ~(a - 1);
}

static inline uint64_t align_down_u64(uint64_t v, uint64_t a) {
    return v & ~(a - 1);
}

#define PAGE_FRAMES_STACK_CAPACITY 0x100000 // up to 1024^2 page frames

static uintptr_t page_frames[PAGE_FRAMES_STACK_CAPACITY];
static size_t page_frames_count;

static void push_page_frame(const uintptr_t page_frame) {
    if (page_frames_count == PAGE_FRAMES_STACK_CAPACITY) {
        return;
    }

    page_frames[page_frames_count++] = page_frame;
}

static uintptr_t pop_page_frame(void) {
    if (page_frames_count == 0) {
        emergency_console_println("Cannot pop from empty stack!");
        return 0; // NULL
    }

    return page_frames[--page_frames_count];
}

void pmm_init(struct limine_memmap_response *memmap) {
    page_frames_count = 0;
    if (memmap == NULL) {
        early_console_println("Got NULL ptr as memmap response; cannot initialize PPM");
        return;
    }

    for (size_t i = 0; i < memmap->entry_count; i++) {
        const struct limine_memmap_entry *memmap_entry = memmap->entries[i];
        if (memmap_entry == NULL) {
            early_console_println("Got NULL ptr as memmap entry, continuing");
            continue;
        }

        if (memmap_entry->type != LIMINE_MEMMAP_USABLE) {
            continue;
        }

        const uint64_t start = align_up_u64(memmap_entry->base, PPM_PAGE_SIZE);
        const uint64_t end = align_down_u64(memmap_entry->base + memmap_entry->length, PPM_PAGE_SIZE);
        for (uint64_t addr = start; addr < end; addr += PPM_PAGE_SIZE) {
            push_page_frame(addr);
        }
    }

    early_console_print("Initialized physical memory manager with ");
    early_console_print_hex_u64(page_frames_count);
    early_console_println(" page frames");
}

uintptr_t pmm_alloc_frame(void) {
    return pop_page_frame();
}

bool pmm_free_frame(uintptr_t physical_frame_address) {
    if (physical_frame_address == 0x0) {
        emergency_console_println("Trying to free NULL ptr as physical frame address!");
        return false;
    }

    if (physical_frame_address % PPM_PAGE_SIZE != 0) {
        emergency_console_println("Trying to free non-aligned physical frame address!");
        return false;
    }

    push_page_frame(physical_frame_address);
    return true;
}

size_t pmm_get_available_frames_count(void) {
    return page_frames_count;
}
