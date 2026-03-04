#pragma once

#include <stdint.h>

void lapic_init(uintptr_t phys_addr);

void lapic_eoi(void);

uint32_t lapic_read(uint32_t reg);

void lapic_write(uint32_t reg, uint32_t value);

// Offsets - simplified for byte-addressing in the internal implementation
#define LAPIC_REG_EOI      0x0B0
#define LAPIC_REG_SVR      0x0F0
#define LAPIC_REG_LVT_TMR  0x320
#define LAPIC_REG_TICRET   0x380
#define LAPIC_REG_TCCR     0x390
#define LAPIC_REG_TDCR     0x3E0
