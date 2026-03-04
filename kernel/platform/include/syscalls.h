#pragma once

#include <stdint.h>
#include "interrupts.h"

struct syscall_frame {
    uint64_t num;
    uint64_t a[5];
    struct interrupt_frame **interrupt_frame;
};

typedef void (*syscall_handler_t)(struct syscall_frame *frame, void *arg);

void syscalls_init(syscall_handler_t syscall_handler, void *syscall_handler_arg);

struct syscall_args {
    uint64_t num;
    uint64_t a[5];
};

uint64_t syscalls_raw(struct syscall_args args);
