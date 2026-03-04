#include "tasks.h"

#include "cpu_interrupts.h"
#include "gdt.h"
#include <string.h>

#include "msr.h"

struct interrupt_frame *task_setup_user(
    const struct vmm_context *user_ctx,
    const uintptr_t entrypoint_vaddr,
    const uintptr_t user_stack_top,
    const uintptr_t kernel_stack_top
) {
    const uintptr_t sp = kernel_stack_top - sizeof(struct interrupt_frame);
    struct interrupt_frame *frame = (struct interrupt_frame *) sp;

    memset(frame, 0, sizeof(struct interrupt_frame));

    frame->ss = USER_DATA_SEGMENT | 3; // 0x1B
    frame->rsp = (user_stack_top & ~0xFULL);
    // @TODO: add some way to specify whether a task should have access to IO ports, and best if only to specific ports
    frame->rflags = 0x202 | (3 << 12); // Interrupts enabled and IO ports allowed
    frame->cs = USER_CODE_SEGMENT | 3; // 0x23
    frame->rip = entrypoint_vaddr;
    // for sysret compatibility
    frame->rcx = entrypoint_vaddr;
    frame->r11 = 0x202;

    frame->cr3 = user_ctx->root;

    return (struct interrupt_frame *) sp;
}

struct interrupt_frame *task_setup_kernel(
    const uintptr_t stack_top,
    const kernel_task_fn_t fn,
    void *arg
) {
    const uintptr_t sp = stack_top - sizeof(struct interrupt_frame);
    struct interrupt_frame *frame = (struct interrupt_frame *) sp;

    memset(frame, 0, sizeof(struct interrupt_frame));

    frame->ss = KERNEL_DATA_SEGMENT;
    frame->rsp = stack_top;
    frame->rflags = 0x202;
    frame->cs = KERNEL_CODE_SEGMENT;
    frame->rip = (uintptr_t) fn;
    frame->rdi = (uintptr_t) arg;
    frame->cr3 = g_kernel_context.root;

    return (struct interrupt_frame *) sp;

}

void task_prepare_switch(const uintptr_t kernel_stack_top, const uint64_t task_id) {
    gdt_set_kernel_stack(kernel_stack_top);
    msr_set_kernel_stack(kernel_stack_top);
    msr_set_task_id(task_id);
}

uint64_t task_get_current_id(void) {
    return msr_get_task_id();
}

void task_set_syscall_return_value(struct interrupt_frame *frame, const uint64_t error_code, const uint64_t value) {
    frame->rax = value;
    frame->rdx = error_code;
}
