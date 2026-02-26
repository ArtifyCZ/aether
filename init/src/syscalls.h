#pragma once

#include <stdint.h>
#include <stddef.h>

struct syscall_args {
    uint64_t num;
    uint64_t a[5];
};

static inline uint64_t syscall_raw(struct syscall_args args) {
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
        "int $0x80"
        : "=a"(ret) // Return value comes back in RAX
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
        : "+r"(x0) // x0 is input (a[0]) and output (ret)
        : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4)
        : "memory" // Barrier to prevent compiler reordering
    );
    ret = x0;
#else
#error "NOT IMPLEMENTED YET"
#endif
    return ret;
}

#define __DO_SYSCALL(number, ...) ({ \
    struct syscall_args _args = { .num = (number), .a = { __VA_ARGS__ } }; \
    syscall_raw(_args); \
})

#define __DEFN_SYSCALL0(name, num) \
    static inline uint64_t name() { \
        return __DO_SYSCALL(num, 0); \
    }

#define __DEFN_SYSCALL1(name, num, type1, arg1) \
    static inline uint64_t name(type1 arg1) { \
        return __DO_SYSCALL(num, (uint64_t)arg1); \
    }

#define __DEFN_SYSCALL2(name, num, type1, arg1, type2, arg2) \
    static inline uint64_t name(type1 arg1, type2 arg2) { \
        return __DO_SYSCALL(num, (uint64_t)arg1, (uint64_t)arg2); \
    }

#define __DEFN_SYSCALL3(name, num, type1, arg1, type2, arg2, type3, arg3) \
    static inline uint64_t name(type1 arg1, type2 arg2, type3 arg3) { \
        return __DO_SYSCALL(num, (uint64_t)arg1, (uint64_t)arg2, (uint64_t)arg3); \
    }

#define __DEFN_SYSCALL4(name, num, type1, arg1, type2, arg2, type3, arg3, type4, arg4) \
    static inline uint64_t name(type1 arg1, type2 arg2, type3 arg3, type4 arg4) { \
        return __DO_SYSCALL(num, (uint64_t)arg1, (uint64_t)arg2, (uint64_t)arg3, (uint64_t)arg4); \
    }

__DEFN_SYSCALL0(sys_exit, 0x00)
__DEFN_SYSCALL3(sys_write, 0x01, int, fd, const void*, buf, size_t, count)
__DEFN_SYSCALL3(sys_clone, 0x02, unsigned long long, flags, const void*, stack_pointer, const void*, entrypoint)
__DEFN_SYSCALL4(sys_mmap, 0x03, unsigned long long, addr, unsigned long long, length, unsigned int, prot, unsigned int, flags)
