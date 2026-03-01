#pragma once

#include <stdint.h>

struct cpu_local_storage {
    uint64_t task_id;
};

void cpu_local_init(void);

struct cpu_local_storage *cpu_local_get(void);
