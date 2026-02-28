#include "interrupts.h"
#include "cpu_interrupts.h"

#include <stdint.h>

#include "boot.h"
#include "gdt.h"
#include "io_wrapper.h"
#include "lapic.h"
#include "drivers/serial.h"

// The IDT structure
struct idt_entry {
    uint16_t isr_low;
    uint16_t kernel_cs;
    uint8_t ist;
    uint8_t attributes;
    uint16_t isr_mid;
    uint32_t isr_high;
    uint32_t reserved;
} __attribute__((packed));

struct idt_ptr {
    uint16_t limit;
    uint64_t base;
} __attribute__((packed));

static struct idt_entry idt[256];
static irq_handler_t handlers[256];
static void *handler_priv[256];

static uint8_t g_interrupt_disable_nesting = 0;

void interrupts_init(void) {
    // This table must be defined in your NASM file (see below)
    extern uint64_t interrupt_stubs[];

    for (int i = 0; i < 256; i++) {
        uint64_t isr = interrupt_stubs[i];

        idt[i].isr_low = (uint16_t) (isr & 0xFFFF);
        idt[i].kernel_cs = KERNEL_CODE_SEGMENT;
        if (i == 0x0E) {
            // Page fault
            idt[i].ist = 1; // Use IST1 stack
        } else {
            idt[i].ist = 0; // No specific stack switching
        }
        uint8_t attributes;
        attributes = 0x8E; // Present, Ring 0, Interrupt Gate
        idt[i].attributes = attributes;
        idt[i].isr_mid = (uint16_t) ((isr >> 16) & 0xFFFF);
        idt[i].isr_high = (uint32_t) ((isr >> 32) & 0xFFFFFFFF);
        idt[i].reserved = 0;
    }

    struct idt_ptr ptr = {
        .limit = sizeof(idt) - 1,
        .base = (uint64_t) &idt
    };

    __asm__ volatile("lidt %0" : : "m"(ptr));
    g_interrupt_disable_nesting = 1;
}

void interrupts_enable(void) {
    g_interrupt_disable_nesting--;
    if (g_interrupt_disable_nesting > 0) return;
    __asm__ volatile("sti");
}

void interrupts_disable(void) {
    g_interrupt_disable_nesting++;
    if (g_interrupt_disable_nesting > 1) return;
    __asm__ volatile("cli");
}

bool interrupts_register_handler(uint32_t irq, irq_handler_t handler, void *priv) {
    if (irq < 0x30 || irq > 0xFF) return false; // allow APIC only (and we are mapping APIC from 0x30 to 0xFF)

    handlers[irq] = handler;
    handler_priv[irq] = priv;

    return true;
}

static void dump_stack(uintptr_t stack_ptr, int qwords) {
    serial_println("--- STACK DUMP ---");
    // Basic sanity check: don't dump if pointer is NULL
    if (stack_ptr < 0x1000) {
        serial_println("RSP is NULL or invalid.");
        return;
    }

    uint64_t* ptr = (uint64_t*)stack_ptr;
    for (int i = 0; i < qwords; i++) {
        serial_print_hex_u64(stack_ptr + (i * 8));
        serial_print(": ");
        serial_print_hex_u64(ptr[i]);
        serial_println("");
    }
}

static void dump_frame(struct interrupt_frame *frame) {
    serial_println("--- REGISTER DUMP ---");
    serial_print("RIP: "); serial_print_hex_u64(frame->rip);
    serial_print(" CS:  "); serial_print_hex_u64(frame->cs);
    serial_print(" RFL: "); serial_print_hex_u64(frame->rflags);
    serial_println("");

    serial_print("RSP: "); serial_print_hex_u64(frame->rsp);
    serial_print(" SS:  "); serial_print_hex_u64(frame->ss);
    serial_print(" CR3: "); serial_print_hex_u64(frame->cr3);
    serial_println("");

    serial_print("RAX: "); serial_print_hex_u64(frame->rax);
    serial_print(" RBX: "); serial_print_hex_u64(frame->rbx);
    serial_print(" RCX: "); serial_print_hex_u64(frame->rcx);
    serial_println("");

    serial_print("RDX: "); serial_print_hex_u64(frame->rdx);
    serial_print(" RDI: "); serial_print_hex_u64(frame->rdi);
    serial_print(" RSI: "); serial_print_hex_u64(frame->rsi);
    serial_println("");

    serial_print("RBP: "); serial_print_hex_u64(frame->rbp);
    serial_print(" R8:  "); serial_print_hex_u64(frame->r8);
    serial_print(" R9:  "); serial_print_hex_u64(frame->r9);
    serial_println("");

    serial_println("---------------------");
}

// The C dispatcher called from NASM
// Returns a pointer to a stack the assembly code should switch to
// If frame pointer == returned address, then the interrupt returns exactly where it was interrupted
uintptr_t x86_64_interrupt_dispatcher(struct interrupt_frame *frame) {
    if (frame->error_code != 0 || frame->interrupt_number < 0x20) {
        // non-zero error code and/or exceptions
        serial_print("Interrupt: ");
        serial_print_hex_u64(frame->interrupt_number);
        serial_println("");
        serial_print("Error code received: ");
        serial_print_hex_u64(frame->error_code);
        serial_println("");
        dump_frame(frame);
        if (frame->interrupt_number == 0x0E) {
            // Page fault
            serial_print("CR2: ");
            uintptr_t cr2;
            // retrieve the CR2 value
            __asm__ volatile("mov %0, %%cr2" : : "r"(cr2));
            serial_print_hex_u64(cr2);
            serial_println("");
        }
        dump_stack(frame->rsp, 16);
        hcf();
    }
    struct interrupt_frame *return_frame = frame;

    if (frame->interrupt_number < 0x30) {
        serial_print("WARNING: legacy PIC interrupt");
        serial_print_hex_u64(frame->interrupt_number);
        serial_println("");
    }

    if (handlers[frame->interrupt_number]) {
        handlers[frame->interrupt_number](&return_frame, handler_priv[frame->interrupt_number]);
    } else {
        serial_print("Unhandled interrupt: ");
        serial_print_hex_u64(frame->interrupt_number);
        serial_println("");
    }

    if (frame->interrupt_number >= 0x20 && frame->interrupt_number <= 0x2F) {
        if (frame->interrupt_number >= 0x28) {
            outb(0xA0, 0x20); // Send EOI to Slave PIC
        }
        outb(0x20, 0x20); // Send EOI to Master PIC
    }

    if (0x30 <= frame->interrupt_number && frame->interrupt_number < 0x80) {
        // APIC interrupts
        lapic_eoi();
    }

    return (uintptr_t) return_frame;
}
