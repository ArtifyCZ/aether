#pragma once

#include "../../include/syscall_user.h"
#include <stdint.h>
#include <stddef.h>

struct syscall_args {
    uint64_t num;
    uint64_t a[5];
};

static inline sys_result_t syscall_raw(struct syscall_args args) {
    sys_err_t err_code;
    uint64_t ret;
#if defined (__x86_64__)
    // Tie C variables to specific registers
    register uint64_t rax __asm__("rax") = args.num;
    register uint64_t rdi __asm__("rdi") = args.a[0];
    register uint64_t rsi __asm__("rsi") = args.a[1];
    register uint64_t rdx __asm__("rdx") = args.a[2];
    register uint64_t r10 __asm__("r10") = args.a[3];
    register uint64_t r8 __asm__("r8") = args.a[4];

    __asm__ volatile (
        "syscall"
        : "=a"(ret), "=d"(err_code)
        : "r"(rax), "r"(rdi), "r"(rsi), "r"(rdx), "r"(r10), "r"(r8)
        : "rcx", "r11", "memory" // Clobbers
    );
#elif defined (__aarch64__)
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
        : "+r"(x0), "+r"(x1)
        : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4)
        : "memory" // Barrier to prevent compiler reordering
    );
    ret = x0;
    err_code = x1;
#else
#error "NOT IMPLEMENTED YET"
#endif
    sys_result_t result = { .err_code = err_code, .value = ret };
    return result;
}

#define __arch_syscall(number, ...) ({ \
    struct syscall_args _args = { .num = (number), .a = { __VA_ARGS__ } }; \
    syscall_raw(_args); \
})

SYSCALLS_LIST(SYSCALL_USER_WRAPPER);

const char *sys_err_get_message(sys_err_t err_code);
