#include "syscalls.h"

#include <stddef.h>
#include "cpu_interrupts.h"

#define SYSCALL_INTERRUPT_VECTOR 0x80

bool syscalls_interrupt_handler(struct interrupt_frame **frame, void *args);

static syscall_handler_t g_syscall_handler = NULL;
static void *g_syscall_handler_arg = NULL;

void syscalls_init(syscall_handler_t syscall_handler, void *syscall_handler_arg) {
    g_syscall_handler = syscall_handler;
    g_syscall_handler_arg = syscall_handler_arg;
    interrupts_register_handler(SYSCALL_INTERRUPT_VECTOR, syscalls_interrupt_handler, NULL);
}

bool syscalls_interrupt_handler(struct interrupt_frame **frame, void *args) {
    (void) args;

    struct syscall_frame syscall_frame = {
        .interrupt_frame = frame,
        .num = (*frame)->rax,
        .a = {
            (*frame)->rdi,
            (*frame)->rsi,
            (*frame)->rdx,
            (*frame)->r10,
            (*frame)->r8,
        },
    };

    struct interrupt_frame *previous_frame = *frame;
    const uint64_t return_value = g_syscall_handler(&syscall_frame, g_syscall_handler_arg);
    previous_frame->rax = return_value;

    return true;
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
        "int $0x80"
        : "=a"(ret) // Return value comes back in RAX
        : "r"(rax), "r"(rdi), "r"(rsi), "r"(rdx), "r"(r10), "r"(r8)
        : "rcx", "r11", "memory" // Clobbers
    );
    return ret;
}
