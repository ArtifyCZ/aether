#include "platform.h"

#include "cpu_local.h"

void platform_init(const struct platform_config *config) {
    (void) config;

    cpu_local_init();
}
