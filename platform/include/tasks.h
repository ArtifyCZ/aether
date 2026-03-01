#pragma once

#include <stdint.h>
#include "virtual_memory_manager.h"

typedef void (*kernel_task_fn_t)(void *arg);

struct interrupt_frame *task_setup_user(
    const struct vmm_context *user_ctx,
    uintptr_t entrypoint_vaddr,
    uintptr_t user_stack_top,
    uintptr_t kernel_stack_top
);

struct interrupt_frame *task_setup_kernel(
    uintptr_t stack_top,
    kernel_task_fn_t fn,
    void *arg
);

void task_prepare_switch(uintptr_t kernel_stack_top, uint64_t task_id);

uint64_t task_get_current_id(void);

void task_set_syscall_return_value(struct interrupt_frame *frame, uint64_t error_code, uint64_t value);
