#pragma once

#include <stdint.h>

#include "interrupts.h"

struct interrupt_frame {
    // Pushed LAST in NASM (lowest address)
    uint64_t cr3;

    uint64_t r15, r14, r13, r12, r11, r10, r9, r8;
    uint64_t rbp, rsi, rdi, rdx, rcx, rbx, rax;
    // ^ rax was pushed FIRST among these, so it is at the highest address here

    // Pushed by the stub macro
    uint64_t interrupt_number;
    uint64_t error_code;

    // Pushed automatically by CPU (highest addresses)
    uint64_t rip, cs, rflags, rsp, ss;
} __attribute__((packed));
