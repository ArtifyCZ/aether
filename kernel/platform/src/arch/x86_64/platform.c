#include "platform.h"

#include "acpi.h"
#include "early_console.h"
#include "gdt.h"
#include "msr.h"

void platform_init(const struct platform_config *config) {
    early_console_println("Setting up GDT...");
    gdt_init();
    early_console_println("GDT initialized!");

    early_console_println("Setting up MSR...");
    msr_init();
    early_console_println("MSR initialized!");

    if (config->rsdp_address != 0x0) {
        early_console_println("Initializing ACPI...");
        acpi_init(config->rsdp_address);
        early_console_println("ACPI initialized!");
    } else {
        early_console_println("No RSDP found!");
    }
}
