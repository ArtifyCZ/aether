#include "interrupts.h"
#include "cpu_interrupts.h"
#include <stdint.h>

#include "gic.h"
#include "stddef.h"
#include "syscalls_priv.h"
#include "drivers/serial.h"

extern void *exception_vector_table;

static irq_handler_t handlers[256];
static void *handler_priv[256];

static uint8_t g_interrupt_disable_nesting = 0;

/**
 * AArch64 ESR_EL1 Exception Class (EC) definitions
 */
#define EC_UNKNOWN          0x00
#define EC_SIMD_FP          0x07
#define EC_SYSCALL          0x15
#define EC_DATA_ABORT_LOWER 0x24
#define EC_DATA_ABORT_SAME  0x25
#define EC_INST_ABORT_LOWER 0x20
#define EC_INST_ABORT_SAME  0x21
#define EC_ALIGNED_FAULT    0x26

void interrupts_init(void) {
    // 1. Install the vector table
    uintptr_t vbar = (uintptr_t) &exception_vector_table;
    __asm__ volatile("msr vbar_el1, %0" : : "r"(vbar));

    // 2. Enable Floating Point/SIMD (to prevent CPACR traps)
    uint64_t cpacr;
    __asm__ volatile("mrs %0, cpacr_el1" : "=r"(cpacr));
    cpacr |= (3 << 20);
    __asm__ volatile("msr cpacr_el1, %0" : : "r"(cpacr));

    // 3. Clear handler table
    for (int i = 0; i < 256; i++) {
        handlers[i] = NULL;
        handler_priv[i] = NULL;
    }

    gic_init();
    g_interrupt_disable_nesting = 1;

    serial_println("AArch64: Interrupt system initialized.");
}

void interrupts_enable(void) {
    g_interrupt_disable_nesting--;
    if (g_interrupt_disable_nesting > 0) return;
    __asm__ volatile("msr daifclr, #2" ::: "memory");
}

void interrupts_disable(void) {
    g_interrupt_disable_nesting++;
    if (g_interrupt_disable_nesting > 1) return;
    __asm__ volatile("msr daifset, #2" ::: "memory");
}

bool interrupts_register_handler(uint32_t irq, irq_handler_t handler, void *priv) {
    if (irq >= 256) return false;

    // Check if already registered (optional safety)
    if (handlers[irq] != NULL) return false;

    handlers[irq] = handler;
    handler_priv[irq] = priv;

    gic_enable_interrupt(irq);

    // Note: In a full implementation, you would talk to the GIC here
    // to unmask the IRQ and set its priority.
    return true;
}

bool interrupts_unregister_handler(uint32_t irq) {
    if (irq >= 256) return false;
    handlers[irq] = NULL;
    handler_priv[irq] = NULL;
    return true;
}

void interrupts_configure_irq(uint32_t irq, irq_type_t type, uint8_t priority) {
    // This is a stub for GIC configuration.
    // Example: GICD_ITARGETSR, GICD_ICFGR (for edge/level), GICD_IPRIORITYR
}

// Helper to dump registers on panic
static void dump_frame(struct interrupt_frame *frame) {
    serial_println("Register Dump:");
    for (int i = 0; i < 31; i++) {
        serial_print("x");
        serial_print_hex_u64(i);
        serial_print(": ");
        serial_print_hex_u64(frame->x[i]);
        if (i % 2 == 1) serial_println("");
        else serial_print("  ");
    }
    serial_println("");
}

// This is called by sync_handler_stub in vectors.S
// Returns a pointer to a stack the assembly code should switch to
// If the frame pointer == returned address, then the interrupt returns exactly where it was interrupted
uintptr_t handle_sync_exception(struct interrupt_frame *frame) {
    uint32_t ec = (frame->esr >> 26) & 0x3F;

    if (ec == EC_SYSCALL) {
        struct interrupt_frame *return_frame = frame;
        syscalls_interrupt_handler(&return_frame);
        return (uintptr_t) return_frame;
    }

    uint64_t far;
    __asm__ volatile("mrs %0, far_el1" : "=r"(far));

    serial_println("------------------------------------------");
    serial_println("!!! KERNEL PANIC: Synchronous Abort !!!");

    serial_print("ESR_EL1: ");
    serial_print_hex_u64(frame->esr);
    serial_print(" (EC: ");
    serial_print_hex_u64(ec);
    serial_println(")");

    serial_print("ELR_EL1 (PC): ");
    serial_print_hex_u64(frame->elr);
    serial_println("");
    serial_print("FAR_EL1 (Addr): ");
    serial_print_hex_u64(far);
    serial_println("");

    switch (ec) {
        case EC_DATA_ABORT_SAME:
        case EC_DATA_ABORT_LOWER:
            serial_println("Reason: Data Abort (Check FAR for faulting address)");
            break;
        case EC_INST_ABORT_SAME:
        case EC_INST_ABORT_LOWER:
            serial_println("Reason: Instruction Abort (Check ELR for faulting PC)");
            break;
        case EC_SIMD_FP:
            serial_println("Reason: Access to SIMD/FP register trapped");
            break;
        case EC_ALIGNED_FAULT:
            serial_println("Reason: Misaligned memory access");
            break;
        default:
            serial_println("Reason: Unknown/Unhandled Exception Class");
            break;
    }

    dump_frame(frame);
    serial_println("------------------------------------------");

    while (1) { __asm__("wfi"); }
}

// Returns a pointer to a stack the assembly code should switch to
// If the frame pointer == returned address, then the interrupt returns exactly where it was interrupted
uintptr_t handle_irq_exception(struct interrupt_frame *frame) {
    const uint32_t irq_id = gic_acknowledge_interrupt();

    struct interrupt_frame *return_frame = frame;
    if (irq_id < 256 && handlers[irq_id]) {
        handlers[irq_id](&return_frame, handler_priv[irq_id]);
    } else {
        serial_print("Unhandled IRQ: ");
        serial_print_hex_u64(irq_id);
        serial_println("");
    }

    gic_end_of_interrupt(irq_id);

    return (uintptr_t) return_frame;
}

// Fast Interrupts (FIQ) - Usually reserved for secure monitor or high-priority tasks
void handle_fiq_exception(struct interrupt_frame *frame) {
    serial_println("Caught FIQ!");
}

// System Errors (SERROR) - Usually asynchronous hardware errors (bad bus access)
void handle_serror_exception(struct interrupt_frame *frame) {
    serial_println("!!! KERNEL PANIC: SError (System Error) !!!");
    dump_frame(frame);
    while (1) { __asm__("wfi"); }
}
