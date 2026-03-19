#pragma once

#include <stdint.h>

void msr_set_kernel_stack(uint64_t stack);

uint64_t msr_get_task_id(void);

void msr_set_task_id(uint64_t task_id);
