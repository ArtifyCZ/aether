#pragma once

#include <stdint.h>

#include "interrupts.h"

struct interrupt_frame {
    uint64_t x[31];
    uint64_t sp_el0;
    uint64_t ttbr0;
    uint64_t spsr;
    uint64_t elr;
    uint64_t esr; // Helpful for debugging sync aborts
} __attribute__((aligned(16)));
