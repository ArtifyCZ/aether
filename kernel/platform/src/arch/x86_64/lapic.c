#include "lapic.h"
#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

static volatile uint8_t *g_lapic_base = NULL;

void lapic_init(uintptr_t phys_addr) {
    if (g_lapic_base == NULL) {
        uintptr_t virt_addr = vaa_alloc_range(VMM_PAGE_SIZE);
        vmm_map_page(
            &g_kernel_context,
            virt_addr,
            phys_addr,
            VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE
        );
        g_lapic_base = (uint8_t *) virt_addr;
    }
}

void lapic_write(uint32_t reg, uint32_t value) {
    *(volatile uint32_t *) (g_lapic_base + reg) = value;
}

uint32_t lapic_read(uint32_t reg) {
    return *(volatile uint32_t *) (g_lapic_base + reg);
}

void lapic_eoi(void) {
    lapic_write(LAPIC_REG_EOI, 0);
}
