#include "gic.h"

#include <stdint.h>

#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

// Simplified GICv2 initialization for QEMU virt
#define GIC_DIST_BASE 0x08000000
#define GIC_CPU_BASE  0x08010000

// 0x00-0x0F are used for software-generated interrupts
// 0x10-0x1F are used for private-peripheral interrupts
// 0x20+ are used for shared peripheral interrupts
#define IRQ_INTID_OFFSET 0x20

static uint64_t g_gic_dist_base = 0;

static uint64_t g_gic_cpu_base = 0;

void gic_init(void) {
    g_gic_dist_base = vaa_alloc_range(VMM_PAGE_SIZE);
    g_gic_cpu_base = vaa_alloc_range(VMM_PAGE_SIZE);

    vmm_map_page(
        &g_kernel_context,
        g_gic_dist_base,
        GIC_DIST_BASE,
        VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE
    );

    vmm_map_page(
        &g_kernel_context,
        g_gic_cpu_base,
        GIC_CPU_BASE,
        VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE
    );

    // 1. Disable Distributor while configuring
    *(volatile uint32_t *) (g_gic_dist_base + 0x000) = 0x0;

    // 2. Mask all interrupts initially (assuming 256 max for now)
    for (int i = 0; i < 256 / 32; i++) {
        *(volatile uint32_t *)(g_gic_dist_base + 0x180 + (i * 4)) = 0xFFFFFFFF;
    }

    // 3. Set all interrupts to Group 1 (standard IRQs)
    for (int i = 0; i < 256 / 32; i++) {
        *(volatile uint32_t *)(g_gic_dist_base + 0x080 + (i * 4)) = 0xFFFFFFFF;
    }

    // 4. Enable Distributor and CPU Interface
    *(volatile uint32_t *) (g_gic_dist_base + 0x000) = 0x3;
    *(volatile uint32_t *) (g_gic_cpu_base + 0x000) = 0x1F;
    *(volatile uint32_t *) (g_gic_cpu_base + 0x004) = 0xF0; // Priority mask
}

// Helper to set priority without grouped configuration logic
void gic_set_priority(uint32_t vector, uint8_t priority) {
    uint32_t prio_reg = vector / 4;
    uint32_t prio_off = (vector % 4) * 8;

    volatile uint32_t *reg = (uint32_t *)(g_gic_dist_base + 0x400 + (prio_reg * 4));
    uint32_t val = *reg;
    val &= ~(0xFF << prio_off);
    val |= (priority << prio_off);
    *reg = val;
}

void gic_mask_vector(const uint32_t intid) {
    // GICD_ICENABLERn (Interrupt Clear-Enable Registers)
    // Offset: 0x180 + (reg * 4)
    const uint32_t reg = intid / 32;
    const uint32_t bit = intid % 32;

    // Writing a 1 to a bit in ICENABLER disables the corresponding interrupt.
    // Writing 0 has no effect.
    *(volatile uint32_t *) (g_gic_dist_base + 0x180 + (reg * 4)) = (1 << bit);
}

void gic_unmask_vector(const uint32_t intid) {
    // GICD_ISENABLERn (Interrupt Set-Enable Registers)
    // Offset: 0x100 + (reg * 4)
    const uint32_t reg = intid / 32;
    const uint32_t bit = intid % 32;

    // Writing a 1 to a bit in ISENABLER enables the corresponding interrupt.
    // Writing 0 has no effect.
    *(volatile uint32_t *) (g_gic_dist_base + 0x100 + (reg * 4)) = (1 << bit);

    // Ensure priority is set to something "runnable" (e.g., 0xA0)
    // Higher numbers are lower priority in GIC.
    gic_set_priority(intid, 0xA0);
}

void gic_configure_interrupt(uint32_t vector, uint8_t priority) {
    // 1. Set Priority (Keep your existing logic)
    uint32_t prio_reg = vector / 4;
    uint32_t prio_off = (vector % 4) * 8;
    uint32_t val = *(volatile uint32_t *) (g_gic_dist_base + 0x400 + (prio_reg * 4));
    val &= ~(0xFF << prio_off);
    val |= (priority << prio_off);
    *(volatile uint32_t *) (g_gic_dist_base + 0x400 + (prio_reg * 4)) = val;

    // 2. Set Group 1 (Standard IRQ routing)
    uint32_t group_reg = vector / 32;
    uint32_t group_bit = vector % 32;
    *(volatile uint32_t *) (g_gic_dist_base + 0x080 + (group_reg * 4)) |= (1 << group_bit);
}

void gic_set_trigger_mode(uint32_t vector, bool edge) {
    // GICD_ICFGRn (Interrupt Configuration Registers)
    // 2 bits per interrupt. Bit [1] = 0 (Level), 1 (Edge)
    uint32_t reg = vector / 16;
    uint32_t bit = (vector % 16) * 2 + 1;

    volatile uint32_t *icfgr = (uint32_t *) (g_gic_dist_base + 0xC00 + (reg * 4));
    if (edge) *icfgr |= (1 << bit);
    else *icfgr &= ~(1 << bit);
}

void gic_set_target_cpu(uint32_t vector, uint8_t cpu_mask) {
    // GICD_ITARGETSRn: 1 byte per interrupt
    // cpu_mask = 0x01 targets CPU 0
    uint32_t reg = vector / 4;
    uint32_t off = (vector % 4) * 8;

    volatile uint32_t* target_reg = (uint32_t*)(g_gic_dist_base + 0x800 + (reg * 4));
    uint32_t val = *target_reg;
    val &= ~(0xFF << off);
    val |= (cpu_mask << off);
    *target_reg = val;
}

uint32_t gic_acknowledge_interrupt(void) {
    return *(volatile uint32_t *) (g_gic_cpu_base + 0x00C) & 0x3FF;
}

void gic_end_of_interrupt(const uint32_t id) {
    *(volatile uint32_t *) (g_gic_cpu_base + 0x010) = id;
}
