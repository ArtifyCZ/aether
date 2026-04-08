#pragma once

#include "kernel/api/syscalls/syscall_errs.h"
#include <stdint.h>
#include <stddef.h>

struct syscall_result {
    enum syscall_err err_code;
    uint64_t value;
};

typedef struct syscall_result sys_result_t;

struct syscall_args {
    uint64_t num;
    uint64_t a[5];
};

__attribute__((noinline))
static sys_result_t syscall_raw(const struct syscall_args *args) {
    // 1. Extract values into standard local variables.
    // This gives the compiler "breathing room" to handle the stack.
    uint64_t n  = args->num;
    uint64_t a0 = args->a[0];
    uint64_t a1 = args->a[1];
    uint64_t a2 = args->a[2];
    uint64_t a3 = args->a[3];
    uint64_t a4 = args->a[4];

    uint64_t ret, err;

#if defined (__x86_64__)
    // 2. Use pinned variables ONLY for registers without direct constraints (r10, r8)
    register uint64_t r10 __asm__("r10") = a3;
    register uint64_t r8  __asm__("r8")  = a4;

    __asm__ volatile (
        "syscall"
        : "=a"(ret), "=d"(err)                   /* Outputs: rax, rdx */
        : "a"(n), "D"(a0), "S"(a1), "d"(a2),     /* Inputs: rax, rdi, rsi, rdx */
          "r"(r10), "r"(r8)                      /* Inputs: r10, r8 (via pinning) */
        : "rcx", "r11", "memory"                 /* Clobbers: syscall destroys rcx/r11 */
    );
#elif defined (__aarch64__)
    register uint64_t x8 __asm__("x8") = n;
    register uint64_t x0 __asm__("x0") = a0;
    register uint64_t x1 __asm__("x1") = a1;
    register uint64_t x2 __asm__("x2") = a2;
    register uint64_t x3 __asm__("x3") = a3;
    register uint64_t x4 __asm__("x4") = a4;

    __asm__ volatile (
        "svc #0"
        : "+r"(x0), "+r"(x1)
        : "r"(x8), "r"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4)
        : "memory"
    );
    ret = x0;
    err = x1;
#endif

    return (sys_result_t){ .err_code = (sys_err_t)err, .value = ret };
}

#define __arch_syscall(number, ...) ({ \
    struct syscall_args _args = { .num = (number), .a = { __VA_ARGS__ } }; \
    syscall_raw(&_args); \
})

#include "libs/libsyscall/syscall_c_stubs.h"

// @TODO: replace all usages and remove this function
static sys_err_t sys_clone(uint64_t flags, const void *sp, const void *ip, uint64_t *out) {
    const uint64_t proc_handle = 0;
    return sys_proc_spawn(proc_handle, flags, (uintptr_t) sp, (uintptr_t) ip, 0x00, out);
}

// @TODO: replace all usages and remove this function
static sys_err_t sys_mmap(uint64_t addr, uint64_t len, uint32_t pr, uint32_t fl, uintptr_t *out) {
    const uint64_t proc_handle = 0;
    return sys_proc_mmap(proc_handle, addr, len, SYS_PROT_READ | SYS_PROT_WRITE, fl, out);
}

const char *sys_err_get_message(sys_err_t err_code);
