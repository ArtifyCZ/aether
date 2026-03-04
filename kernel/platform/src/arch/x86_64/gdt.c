#include "gdt.h"

#include <string.h>

#define GDT_ENTRIES 10
static struct gdt_entry g_gdt[GDT_ENTRIES];
static struct tss_entry g_tss;

static uint8_t g_ist1_stack[8192]; // Dedicated interrupt stack

static void gdt_set_entry(int num, uint32_t base, uint32_t limit, uint8_t access, uint8_t gran) {
    g_gdt[num].base_low = (base & 0xFFFF);
    g_gdt[num].base_middle = (base >> 16) & 0xFF;
    g_gdt[num].base_high = (base >> 24) & 0xFF;

    g_gdt[num].limit_low = (limit & 0xFFFF);
    g_gdt[num].granularity = (limit >> 16) & 0x0F;

    g_gdt[num].granularity |= gran & 0xF0;
    g_gdt[num].access = access;
}

static void gdt_set_tss_descriptor(int num, uint64_t base, uint32_t limit) {
    struct tss_descriptor *tss_desc = (struct tss_descriptor *) &g_gdt[num];

    gdt_set_entry(num, (uint32_t) base, limit, 0x89, 0x40);
    tss_desc->base_upper32 = (uint32_t) (base >> 32);
    tss_desc->reserved = 0;
}

void gdt_init(void) {
    memset(g_gdt, 0, sizeof(g_gdt));
    memset(&g_tss, 0, sizeof(g_tss));

    // Index 0: Null
    // Index 1: Kernel Code (0x08)
    gdt_set_entry(1, 0, 0xFFFFFFFF, 0x9A, 0x20);
    // Index 2: Kernel Data (0x10)
    gdt_set_entry(2, 0, 0xFFFFFFFF, 0x92, 0x00);

    // Index 3: Dummy/User 32-bit Code (0x18) - Required by some Intel sysret implementations
    gdt_set_entry(3, 0, 0xFFFFFFFF, 0xFA, 0x00);

    // Index 4: User Data (0x20) - sysret uses this for SS (STAR[63:48] + 8)
    gdt_set_entry(4, 0, 0xFFFFFFFF, 0xF2, 0x00);

    // Index 5: User Code 64 (0x28) - sysret uses this for CS (STAR[63:48] + 16)
    gdt_set_entry(5, 0, 0xFFFFFFFF, 0xFA, 0x20);

    // Index 6 & 7: TSS (0x30)
    gdt_set_tss_descriptor(6, (uint64_t) &g_tss, sizeof(g_tss) - 1);

    g_tss.ist[0] = (uint64_t) &g_ist1_stack[8192];

    // 0x28: TSS (Occupies two slots: 5 and 6)
    g_tss.iopb_offset = sizeof(struct tss_entry);
    gdt_set_tss_descriptor(6, (uint64_t) &g_tss, sizeof(g_tss) - 1);

    struct {
        uint16_t limit;
        uint64_t base;
    } __attribute__((packed)) gdtr = {
        .limit = sizeof(g_gdt) - 1,
        .base = (uint64_t) &g_gdt,
    };

    __asm__ volatile ("lgdt %0" :: "m" (gdtr));

    // Reload data segments
    __asm__ volatile(
        "mov $0x10, %%ax\n"
        "mov %%ax, %%ds\n"
        "mov %%ax, %%es\n"
        "mov %%ax, %%ss\n"
        "xor %%ax, %%ax\n"
        "mov %%ax, %%fs\n"
        "mov %%ax, %%gs\n"
        // Reload Code Segment (CS) using lretq
        "pushq $0x08\n"         // New CS
        "lea 1f(%%rip), %%rax\n"
        "pushq %%rax\n"         // New RIP
        "lretq\n"               // Pops RIP, then CS
        "1:\n"
        : : : "rax", "memory"
    );

    __asm__ volatile("ltr %%ax" : : "a"(0x30));
}

void gdt_set_kernel_stack(uintptr_t stack) {
    g_tss.rsp0 = stack;
}
