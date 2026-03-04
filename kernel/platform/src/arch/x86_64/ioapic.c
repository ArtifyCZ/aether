#include "ioapic.h"

#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

static volatile uint32_t *g_ioapic_ptr = NULL;

void ioapic_init(uintptr_t phys_addr) {
    uintptr_t virt_addr = vaa_alloc_range(VMM_PAGE_SIZE);

    vmm_map_page(
        &g_kernel_context,
        virt_addr,
        phys_addr,
        VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE
    );

    g_ioapic_ptr = (uint32_t *) virt_addr;
}

uint32_t ioapic_read(uint8_t reg) {
    g_ioapic_ptr[0] = reg; // Write to IOREGSEL
    return g_ioapic_ptr[4]; // Read from IOWIN (0x10 / 4 = 4)
}

void ioapic_write(uint8_t reg, uint32_t value) {
    g_ioapic_ptr[0] = reg; // Write to IOREGSEL
    g_ioapic_ptr[4] = value; // Write to IOWIN
}

void ioapic_set_entry(uint8_t pin, uint32_t vector) {
    uint32_t low_index = 0x10 + (pin * 2);
    uint32_t high_index = low_index + 1;

    // High 32 bits: Destination ID 0 in the top 8 bits
    ioapic_write(high_index, 0x00000000);

    // Low 32 bits: Vector, and unmask (bit 16 = 0)
    // For ISA IRQs (like keyboard), they are usually Active High / Edge Triggered
    // So bits 13 and 15 stay 0.
    ioapic_write(low_index, vector);
}

void ioapic_set_mask(const uint8_t pin, const bool mask) {
    // The Redirection Table entry is 64 bits (two 32-bit registers)
    // Register 0x10 + (pin * 2) is the LOW 32 bits.
    // Bit 16 (Mask) is in the LOW 32 bits.

    uint32_t low_index = 0x10 + (pin * 2);

    uint32_t val = ioapic_read(low_index); // Read current LOW bits
    if (mask) {
        val |= (1 << 16);  // Set bit 16 to MASK
    } else {
        val &= ~(1 << 16); // Clear bit 16 to UNMASK
    }
    ioapic_write(low_index, val);
}
