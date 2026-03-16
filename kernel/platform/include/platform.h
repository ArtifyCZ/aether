#pragma once

struct platform_config {
    void *rsdp_address;
};

void platform_init(const struct platform_config *config);
