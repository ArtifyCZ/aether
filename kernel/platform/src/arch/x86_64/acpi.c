#include "acpi.h"
#include "acpi_priv.h"
#include "boot.h"
#include "ioapic.h"
#include "string.h"

#include "early_console.h"
#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

static volatile struct rsdp_descriptor *g_rsdp = NULL;

static uintptr_t g_ioapic = 0x0;

static void *map_physical_table(uintptr_t phys_addr) {
    uintptr_t page_phys = phys_addr & ~(VMM_PAGE_SIZE - 1);
    uintptr_t offset    = phys_addr & (VMM_PAGE_SIZE - 1);

    // Allocate a virtual range
    uintptr_t virt_base = vaa_alloc_range(VMM_PAGE_SIZE);

    vmm_map_page(&g_kernel_context, virt_base, page_phys,
                 VMM_FLAG_PRESENT | VMM_FLAG_WRITE | VMM_FLAG_DEVICE);

    // Return the virtual address + the original offset
    return (void *)(virt_base + offset);}

static struct acpi_header *acpi_find_table(char *target_signature) {
    // 1. Map the RSDT header first to find out how big it is
    uintptr_t rsdt_phys = (uintptr_t) g_rsdp->rsdt_address;

    // Using a helper to map physical to virtual (you'll need to implement this wrapper)
    struct rsdt *rsdt_ptr = map_physical_table(rsdt_phys);

    // 2. Calculate how many pointers are in the table
    // (Total length - header size) / size of a 32-bit pointer
    uint32_t entries = (rsdt_ptr->header.length - sizeof(struct acpi_header)) / 4;

    for (uint32_t i = 0; i < entries; i++) {
        early_console_print("RSDT entry: ");
        early_console_print_hex_u64(i);
        early_console_println("");
        struct acpi_header *header = map_physical_table(rsdt_ptr->pointer_to_other_tables[i]);

        if (memcmp(header->signature, target_signature, 4) == 0) {
            return (void *) header;
        }
        // If not found, you might want to unmap 'header' here to save virtual space
    }

    return NULL;
}

void acpi_init(void *rsdp) {
    g_rsdp = rsdp;

    if (g_rsdp->revision != 0) {
        early_console_println("==================");
        early_console_println("  KERNEL PANIC");
        early_console_print("RSDP revision: ");
        early_console_print_hex_u64(g_rsdp->revision);
        early_console_println("");
        early_console_println("NOT IMPLEMENTED!");
        hcf();
    }

    struct madt_header *madt = (struct madt_header *)acpi_find_table("APIC");
    if (!madt) {
        early_console_println("MADT not found!");
        return;
    }

    uint8_t *ptr = (uint8_t *)(madt + 1); // Point to first entry after header
    uint8_t *end = (uint8_t *)madt + madt->header.length;

    while (ptr < end) {
        struct madt_entry_header *entry = (struct madt_entry_header *)ptr;

        switch (entry->type) {
            case 1: { // I/O APIC
                struct madt_ioapic *io = (struct madt_ioapic *)ptr;
                early_console_print("Found I/O APIC at phys: ");
                early_console_print_hex_u64(io->address);
                early_console_println("");
                g_ioapic = io->address;
                break;
            }
            case 2: { // Interrupt Source Override
                struct madt_iso *iso = (struct madt_iso *)ptr;
                early_console_print("ISO: IRQ ");
                early_console_print_hex_u64(iso->irq_source);
                early_console_print(" -> GSI ");
                early_console_print_hex_u64(iso->gsi);
                early_console_println("");
                break;
            }
        }
        ptr += entry->length;
    }

    ioapic_init(g_ioapic);
}
