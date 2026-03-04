#include "syscalls.h"

#include <stddef.h>

#include "boot.h"
#include "cpu_interrupts.h"

struct interrupt_frame *syscalls_inner_handler(struct interrupt_frame *frame);

static syscall_handler_t g_syscall_handler = NULL;
static void *g_syscall_handler_arg = NULL;

void syscalls_init(syscall_handler_t syscall_handler, void *syscall_handler_arg) {
    g_syscall_handler = syscall_handler;
    g_syscall_handler_arg = syscall_handler_arg;
}

struct interrupt_frame *syscalls_inner_handler(struct interrupt_frame *frame) {
    struct syscall_frame sf = {
        .interrupt_frame = &frame, // Note: passing address of a pointer
        .num = frame->rax,
        .a = { frame->rdi, frame->rsi, frame->rdx, frame->r10, frame->r8 },
    };

    g_syscall_handler(&sf, g_syscall_handler_arg);

    // Return the (potentially updated) frame pointer
    return frame;
}

uint64_t syscalls_raw(struct syscall_args args) {
    uint64_t ret;
    // Tie C variables to specific registers
    register uint64_t rax __asm__("rax") = args.num;
    register uint64_t rdi __asm__("rdi") = args.a[0];
    register uint64_t rsi __asm__("rsi") = args.a[1];
    register uint64_t rdx __asm__("rdx") = args.a[2];
    register uint64_t r10 __asm__("r10") = args.a[3];
    register uint64_t r8 __asm__("r8") = args.a[4];

    __asm__ volatile (
        "syscall"
        : "=a"(ret) // Return value comes back in RAX
        : "r"(rax), "r"(rdi), "r"(rsi), "r"(rdx), "r"(r10), "r"(r8)
        : "rcx", "r11", "memory" // Clobbers
    );
    return ret;
}
