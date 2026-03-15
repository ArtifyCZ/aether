#pragma once

#include "limine.h"

struct platform_config {
    struct limine_framebuffer *framebuffer;
    struct limine_module_response *modules;
    void *rsdp_address;
};

void platform_init(const struct platform_config *config);
