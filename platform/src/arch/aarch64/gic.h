// Generic interrupt controller on aarch64

#pragma once

#include <stdbool.h>
#include <stdint.h>

void gic_init(void);

void gic_mask_vector(uint32_t intid);

void gic_unmask_vector(uint32_t intid);

void gic_configure_interrupt(uint32_t vector, uint8_t priority);

void gic_set_trigger_mode(uint32_t vector, bool edge);

void gic_set_target_cpu(uint32_t vector, uint8_t cpu_mask);

uint32_t gic_acknowledge_interrupt(void);

void gic_end_of_interrupt(uint32_t id);
