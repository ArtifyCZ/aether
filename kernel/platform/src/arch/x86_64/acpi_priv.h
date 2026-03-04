#pragma once

#include "acpi.h"

#include <stdint.h>

struct rsdp_descriptor {
    char signature[8];
    uint8_t checksum;
    char oem_id[6];
    uint8_t revision;
    uint32_t rsdt_address;      // 32-bit pointer (for ACPI 1.0)

    // The following fields only exist if revision >= 2 (ACPI 2.0+)
    uint32_t length;
    uint64_t xsdt_address;      // 64-bit pointer! This is what we want.
    uint8_t extended_checksum;
    uint8_t reserved[3];
} __attribute__((packed));

struct acpi_header {
    char signature[4];
    uint32_t length;
    uint8_t revision;
    uint8_t checksum;
    char oem_id[6];
    char oem_table_id[8];
    uint32_t oem_revision;
    uint32_t creator_id;
    uint32_t creator_revision;
} __attribute__((packed));

struct rsdt {
    struct acpi_header header;
    uint32_t pointer_to_other_tables[]; // Array of 32-bit physical pointers
} __attribute__((packed));

struct madt_header {
    struct acpi_header header;
    uint32_t lapic_addr;      // Local APIC physical address
    uint32_t flags;           // 1 = Dual 8259s installed (Legacy PIC)
} __attribute__((packed));

struct madt_entry_header {
    uint8_t type;
    uint8_t length;
} __attribute__((packed));

// Type 1: I/O APIC
struct madt_ioapic {
    struct madt_entry_header header;
    uint8_t ioapic_id;
    uint8_t reserved;
    uint32_t address;         // Physical address of this I/O APIC
    uint32_t gsiv_base;       // Global System Interrupt Vector Base
} __attribute__((packed));

// Type 2: Interrupt Source Override
struct madt_iso {
    struct madt_entry_header header;
    uint8_t bus_source;       // Usually 0 (ISA)
    uint8_t irq_source;       // The bus-relative IRQ (e.g., 1 for keyboard)
    uint32_t gsi;             // The I/O APIC pin it's wired to
    uint16_t flags;           // Polarity and Trigger Mode
} __attribute__((packed));
