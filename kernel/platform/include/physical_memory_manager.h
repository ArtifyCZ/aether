#pragma once

#define PPM_PAGE_SIZE 0x1000 // 4 KiB


#include <limine.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

void pmm_init(struct limine_memmap_response *memmap);

/**
 * @return physical page address (4KiB frame); NULL if out of available of physical frames
 */
uintptr_t pmm_alloc_frame(void);

bool pmm_free_frame(uintptr_t physical_frame_address);

size_t pmm_get_available_frames_count(void);
