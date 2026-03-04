#include "boot.h"

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <limine.h>

#include "early_console.h"
#include "emergency_console.h"
#include "interrupts.h"
#include "modules.h"
#include "physical_memory_manager.h"
#include "psf.h"
#include "terminal.h"
#include "virtual_address_allocator.h"
#include "virtual_memory_manager.h"

// Set the base revision to 4, this is recommended as this is the latest
// base revision described by the Limine boot protocol specification.
// See specification for further info.

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile uint64_t limine_base_revision[] = LIMINE_BASE_REVISION(4);

// The Limine requests can be placed anywhere, but it is important that
// the compiler does not optimise them away, so, usually, they should
// be made volatile or equivalent, _and_ they should be accessed at least
// once or marked as used with the "used" attribute as done here.

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST_ID,
    .revision = 2
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_stack_size_request stack_size_request = {
    .id = LIMINE_STACK_SIZE_REQUEST_ID,
    .stack_size = 0x080000, // 128 kB
    .revision = 0
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST_ID,
    .revision = 0
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_hhdm_request hhdm_request = {
    .id = LIMINE_HHDM_REQUEST_ID,
    .revision = 0
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_executable_address_request kernel_address_request = {
    .id = LIMINE_EXECUTABLE_ADDRESS_REQUEST_ID,
    .revision = 0
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_module_request module_request = {
    .id = LIMINE_MODULE_REQUEST_ID,
    .revision = 0
};

__attribute__((used, section(".limine_requests"), aligned(8)))
static volatile struct limine_rsdp_request rsdp_request = {
    .id = LIMINE_RSDP_REQUEST_ID,
    .revision = 0
};

uint64_t g_hhdm_offset;


// Finally, define the start and end markers for the Limine requests.
// These can also be moved anywhere, to any .c file, as seen fit.

__attribute__((used, section(".limine_requests_start"), aligned(8)))
static volatile uint64_t limine_requests_start_marker[] = LIMINE_REQUESTS_START_MARKER;

__attribute__((used, section(".limine_requests_end"), aligned(8)))
static volatile uint64_t limine_requests_end_marker[] = LIMINE_REQUESTS_END_MARKER;

// Halt and catch fire function.
_Noreturn void hcf(void) {
    interrupts_disable(); // prevent any switches

    for (;;) {
#if defined (__x86_64__)
        asm ("hlt");
#elif defined (__aarch64__) || defined (__riscv)
        asm ("wfi");
#elif defined (__loongarch64)
        asm ("idle 0");
#endif
    }
}

void try_virtual_mapping(void) {
    const uintptr_t physical_frame = pmm_alloc_frame();
    if (physical_frame == 0x0) {
        early_console_println("Cannot allocate physical frame for virtual mapping!");
        return;
    }
    const uintptr_t virtual_address = 0xFFFFD00000000000;
    if (vmm_translate(&g_kernel_context, virtual_address) != 0x0) {
        early_console_println("Cannot map virtual address 0xFFFFC00000000000, address already mapped!");
        return;
    }

    if (!vmm_map_page(&g_kernel_context, virtual_address, physical_frame, VMM_FLAG_PRESENT | VMM_FLAG_WRITE)) {
        early_console_println("Cannot map virtual address 0xFFFFC00000000000!");
        return;
    }

    early_console_println("Virtual mapping successful!");
    early_console_println("Trying to write to the mapped memory");

    volatile uint64_t *ptr = (volatile uint64_t *) virtual_address;
    ptr[0] = 0x1122334455667788ull;
    ptr[1] = 0xA5A5A5A5A5A5A5A5ull;

    early_console_println("Done!");

    if (ptr[0] != 0x1122334455667788ull || ptr[1] != 0xA5A5A5A5A5A5A5A5ull) {
        early_console_println("VMM test: readback mismatch");
        (void) vmm_unmap_page(&g_kernel_context, virtual_address);
        pmm_free_frame(physical_frame);
        return;
    }

    (void) vmm_unmap_page(&g_kernel_context, virtual_address);
    pmm_free_frame(physical_frame);

    early_console_println("VMM test: success!");
}

__attribute__((used)) void boot(void) {
    // Ensure the bootloader actually understands our base revision (see spec).
    if (LIMINE_BASE_REVISION_SUPPORTED(limine_base_revision) == false) {
        hcf();
    }

    // Ensure we got a framebuffer.
    if (framebuffer_request.response == NULL
        || framebuffer_request.response->framebuffer_count < 1) {
        hcf();
    }

    pmm_init(memmap_request.response);
    vaa_init();

    if (hhdm_request.response == NULL) {
        hcf();
    }
    g_hhdm_offset = hhdm_request.response->offset;

    vmm_init(hhdm_request.response->offset);

    uintptr_t serial_device_base;
#if defined (__x86_64__)
    serial_device_base = 0x3f8;
#elif defined (__aarch64__)
    serial_device_base = 0x9000000;
#else
#error "Not implemented yet"
#endif
    early_console_init(serial_device_base);
    early_console_println("Early console initialized!");
    early_console_println("");
    early_console_println("Booting...");

    if (memmap_request.response == NULL) {
        early_console_println("Limine memmap missing; cannot init PMM");
        hcf();
    }

    if (hhdm_request.response == NULL) {
        early_console_println("Limine HHDM missing; cannot init VMM");
        hcf();
    }

    try_virtual_mapping();

    modules_init(module_request.response);

    const struct limine_file *font = module_find("kernel-font.psf");

    if (font != NULL) {
        psf_init(font->address, font->size, framebuffer_request.response->framebuffers[0]);
        terminal_init(framebuffer_request.response->framebuffers[0]);
    } else {
        early_console_println("Could not find kernel-font.psf!");
    }

    terminal_set_foreground_color(0xD4DBDF);
    terminal_set_background_color(0x04121B);
    terminal_clear();

    terminal_println("Hello world!");

    kernel_main(g_hhdm_offset, (uintptr_t) rsdp_request.response->address);

    emergency_console_println("=== KERNEL PANIC ===");
    emergency_console_println("kernel_main function has returned");
    hcf();
}
