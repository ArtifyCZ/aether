#include "platform.h"

#include "acpi.h"
#include "gdt.h"
#include "msr.h"
#include "drivers/serial.h"

void platform_init(const struct platform_config *config) {
    serial_println("Setting up GDT...");
    gdt_init();
    serial_println("GDT initialized!");

    serial_println("Setting up MSR...");
    msr_init();
    serial_println("MSR initialized!");

    if (config->rsdp_address != 0x0) {
        serial_println("Initializing ACPI...");
        acpi_init(config->rsdp_address);
        serial_println("ACPI initialized!");
    } else {
        serial_println("No RSDP found!");
    }
}
