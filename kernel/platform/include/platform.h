#pragma once

#include "limine.h"
#include <stdint.h>

struct platform_config {
    uint64_t hhdm_offset;
    struct limine_framebuffer *framebuffer;
    struct limine_module_response *modules;
    void *rsdp_address;
};

void platform_init(const struct platform_config *config);
