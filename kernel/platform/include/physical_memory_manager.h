// NOTE THAT THE IMPLEMENTATION HAS BEEN MOVED TO RUST!

#pragma once

#define PPM_PAGE_SIZE 0x1000 // 4 KiB

#include <limine.h>
#include <stdbool.h>
#include <stdint.h>

/**
 * @return physical page address (4KiB frame); NULL if out of available of
 * physical frames
 */
uintptr_t pmm_alloc_frame(void);
