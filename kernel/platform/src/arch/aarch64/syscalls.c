#pragma once

#include "syscalls.h"
#include "syscalls_priv.h"

#include "cpu_interrupts.h"
#include <stdbool.h>
#include <stddef.h>

static syscall_handler_t g_syscall_handler = NULL;
static void *g_syscall_handler_arg = NULL;

void syscalls_init(syscall_handler_t syscall_handler, void *syscall_handler_arg) {
    g_syscall_handler = syscall_handler;
    g_syscall_handler_arg = syscall_handler_arg;
}

bool syscalls_interrupt_handler(struct interrupt_frame **frame) {
    struct syscall_frame syscall_frame = {
        .interrupt_frame = frame,
        .num = (*frame)->x[8],
        .a = {
            (*frame)->x[0],
            (*frame)->x[1],
            (*frame)->x[2],
            (*frame)->x[3],
            (*frame)->x[4],
        },
    };

    g_syscall_handler(&syscall_frame, g_syscall_handler_arg);

    return true;
}

uint64_t syscalls_raw(struct syscall_args args) {
    uint64_t ret;
    // x8 is the syscall number
    register uint64_t x8 __asm__("x8") = args.num;

    // x0-x4 are the arguments
    register uint64_t x0 __asm__("x0") = args.a[0];
    register uint64_t x1 __asm__("x1") = args.a[1];
    register uint64_t x2 __asm__("x2") = args.a[2];
    register uint64_t x3 __asm__("x3") = args.a[3];
    register uint64_t x4 __asm__("x4") = args.a[4];

    __asm__ volatile (
        "svc #0"
        : "+r"(x0) // x0 is input (a[0]) and output (ret)
        : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4)
        : "memory" // Barrier to prevent compiler reordering
    );
    ret = x0;
    return ret;
}
