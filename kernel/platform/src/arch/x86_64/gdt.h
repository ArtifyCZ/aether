#pragma once

#include <stdint.h>

#define KERNEL_CODE_SEGMENT 0x08
#define KERNEL_DATA_SEGMENT 0x10
#define USER_DATA_SEGMENT 0x20
#define USER_CODE_SEGMENT 0x28

struct gdt_entry {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t base_middle;
    uint8_t access;
    uint8_t granularity;
    uint8_t base_high;
} __attribute__((packed));

struct tss_descriptor {
    struct gdt_entry low;
    uint32_t base_upper32;
    uint32_t reserved;
} __attribute__((packed));

struct tss_entry {
    uint32_t reserved0;
    uint64_t rsp0; // Stack pointer for Ring 0
    uint64_t rsp1;
    uint64_t rsp2;
    uint64_t reserved1;
    uint64_t ist[7]; // Interrupt Stack Table
    uint64_t reserved2;
    uint16_t reserved3;
    uint16_t iopb_offset;
} __attribute__((packed));

void gdt_init(void);

void gdt_set_kernel_stack(uintptr_t stack);
