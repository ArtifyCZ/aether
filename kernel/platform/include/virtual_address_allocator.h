#pragma once

#include <stdint.h>
#include <stddef.h>

void vaa_init(void);

uintptr_t vaa_alloc_range(size_t size);
