#include "platform.h"

#include <stddef.h>

#include "acpi.h"
#include "boot.h"
#include "early_console.h"
#include "gdt.h"
#include "modules.h"
#include "msr.h"
#include "psf.h"
#include "terminal.h"
#include "virtual_memory_manager.h"

void platform_init(const struct platform_config *config) {
    vmm_init(config->hhdm_offset);

    const uintptr_t serial_device_base = 0x3f8;
    early_console_init(serial_device_base);
    early_console_println("Early console initialized!");
    early_console_println("");
    early_console_println("Booting...");

    modules_init(config->modules);

    const struct limine_file *font = module_find("kernel-font.psf");

    if (font != NULL) {
        psf_init(font->address, font->size, config->framebuffer);
        terminal_init(config->framebuffer);
    } else {
        early_console_println("Could not find kernel-font.psf!");
    }

    terminal_set_foreground_color(0xD4DBDF);
    terminal_set_background_color(0x04121B);
    terminal_clear();

    terminal_println("Hello world!");

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
