#pragma once

#include <stddef.h>
#include <stdint.h>

struct boot_info {
    void *initrd_start;
    size_t initrd_size;
};
