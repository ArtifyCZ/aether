#include "platform.h"

#include "cpu_local.h"
#include "terminal.h"
#include <stddef.h>
#include "boot.h"
#include "early_console.h"
#include "modules.h"
#include "psf.h"
#include "virtual_memory_manager.h"

void platform_init(const struct platform_config *config) {
    (void) config;

    vmm_init(config->hhdm_offset);

    const uintptr_t serial_device_base = 0x9000000;
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

    cpu_local_init();
}
