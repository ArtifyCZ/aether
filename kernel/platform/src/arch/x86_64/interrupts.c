#include "interrupts.h"
#include "cpu_interrupts.h"

#include "emergency_console.h"
#include <stddef.h>
#include <stdint.h>
#include "boot.h"
#include "gdt.h"
#include "ioapic.h"
#include "io_wrapper.h"
#include "lapic.h"

// 0x00 <= n < 0x20 are CPU exceptions
// 0x20 <= n < 0x30 usually are the legacy PIC interrupt vectors
// 0x30 <= n < 0xFF should be free to use for APIC
#define IRQ_INTERRUPT_VECTOR_OFFSET 0x30

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

static irq_handler_new_t g_irq_handler;
static void *g_irq_handler_priv;

static uint8_t g_interrupt_disable_nesting = 0;

void interrupts_init(void) {
    // This table must be defined in your NASM file (see below)
    extern uint64_t interrupt_stubs[];
    g_irq_handler = NULL;
    g_irq_handler_priv = NULL;

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

void interrupts_set_irq_handler(irq_handler_new_t handler, void *priv) {
    g_irq_handler = handler;
    g_irq_handler_priv = priv;
}

void interrupts_mask_irq(uint8_t irq) {
    ioapic_set_mask(irq, true);
}

void interrupts_unmask_irq(uint8_t irq) {
    ioapic_set_mask(irq, false);
    ioapic_set_entry(irq, irq + IRQ_INTERRUPT_VECTOR_OFFSET);
}

bool interrupts_register_handler(uint32_t irq, irq_handler_t handler, void *priv) {
    if (irq < IRQ_INTERRUPT_VECTOR_OFFSET || irq > 0xFF) return false; // allow APIC only (and we are mapping APIC from 0x30 to 0xFF)

    handlers[irq] = handler;
    handler_priv[irq] = priv;

    return true;
}

static void dump_stack(uintptr_t stack_ptr, int qwords) {
    emergency_console_println("--- STACK DUMP ---");
    // Basic sanity check: don't dump if pointer is NULL
    if (stack_ptr < 0x1000) {
        emergency_console_println("RSP is NULL or invalid.");
        emergency_console_print("RSP: ");
        emergency_console_print_hex_u64(stack_ptr);
        return;
    }

    uint64_t* ptr = (uint64_t*)stack_ptr;
    for (int i = 0; i < qwords; i++) {
        emergency_console_print_hex_u64(stack_ptr + (i * 8));
        emergency_console_print(": ");
        emergency_console_print_hex_u64(ptr[i]);
        emergency_console_println("");
    }
}

static void dump_frame(struct interrupt_frame *frame) {
    emergency_console_println("--- REGISTER DUMP ---");
    emergency_console_print("RIP: "); emergency_console_print_hex_u64(frame->rip);
    emergency_console_print(" CS:  "); emergency_console_print_hex_u64(frame->cs);
    emergency_console_print(" RFL: "); emergency_console_print_hex_u64(frame->rflags);
    emergency_console_println("");

    emergency_console_print("RSP: "); emergency_console_print_hex_u64(frame->rsp);
    emergency_console_print(" SS:  "); emergency_console_print_hex_u64(frame->ss);
    emergency_console_print(" CR3: "); emergency_console_print_hex_u64(frame->cr3);
    emergency_console_println("");

    emergency_console_print("RAX: "); emergency_console_print_hex_u64(frame->rax);
    emergency_console_print(" RBX: "); emergency_console_print_hex_u64(frame->rbx);
    emergency_console_print(" RCX: "); emergency_console_print_hex_u64(frame->rcx);
    emergency_console_println("");

    emergency_console_print("RDX: "); emergency_console_print_hex_u64(frame->rdx);
    emergency_console_print(" RDI: "); emergency_console_print_hex_u64(frame->rdi);
    emergency_console_print(" RSI: "); emergency_console_print_hex_u64(frame->rsi);
    emergency_console_println("");

    emergency_console_print("RBP: "); emergency_console_print_hex_u64(frame->rbp);
    emergency_console_print(" R8:  "); emergency_console_print_hex_u64(frame->r8);
    emergency_console_print(" R9:  "); emergency_console_print_hex_u64(frame->r9);
    emergency_console_println("");

    emergency_console_println("---------------------");
}

// The C dispatcher called from NASM
// Returns a pointer to a stack the assembly code should switch to
// If frame pointer == returned address, then the interrupt returns exactly where it was interrupted
uintptr_t x86_64_interrupt_dispatcher(struct interrupt_frame *frame) {
    if (frame->error_code != 0 || frame->interrupt_number < 0x20) {
        // non-zero error code and/or exceptions
        emergency_console_print("Interrupt: ");
        emergency_console_print_hex_u64(frame->interrupt_number);
        emergency_console_println("");
        emergency_console_print("Error code received: ");
        emergency_console_print_hex_u64(frame->error_code);
        emergency_console_println("");
        dump_frame(frame);
        if (frame->interrupt_number == 0x0E) {
            // Page fault
            emergency_console_print("CR2: ");
            uintptr_t cr2;
            // retrieve the CR2 value
            __asm__ volatile("mov %0, %%cr2" : : "r"(cr2));
            emergency_console_print_hex_u64(cr2);
            emergency_console_println("");
        }
        dump_stack(frame->rsp, 16);
        hcf();
    }
    struct interrupt_frame *return_frame = frame;

    if (frame->interrupt_number < IRQ_INTERRUPT_VECTOR_OFFSET) {
        emergency_console_print("KERNEL PANIC: legacy PIC interrupt");
        emergency_console_print_hex_u64(frame->interrupt_number);
        emergency_console_println("");
        hcf();
    }

    if (handlers[frame->interrupt_number]) {
        handlers[frame->interrupt_number](&return_frame, handler_priv[frame->interrupt_number]);
    }

    if (frame->interrupt_number >= IRQ_INTERRUPT_VECTOR_OFFSET) {
        const uint8_t irq = frame->interrupt_number - IRQ_INTERRUPT_VECTOR_OFFSET;
        if (g_irq_handler != NULL && irq != 0x00) {
            // IRQ 0x00 is the LAPIC timer, which we don't want to redirect to the userspace
            g_irq_handler(&return_frame, irq, g_irq_handler_priv);
        }
    }

    if (frame->interrupt_number >= 0x20 && frame->interrupt_number <= 0x2F) {
        if (frame->interrupt_number >= 0x28) {
            outb(0xA0, 0x20); // Send EOI to Slave PIC
        }
        outb(0x20, 0x20); // Send EOI to Master PIC
    }

    if (IRQ_INTERRUPT_VECTOR_OFFSET <= frame->interrupt_number && frame->interrupt_number <= 0xFF) {
        // APIC interrupts
        lapic_eoi();
    }

    return (uintptr_t) return_frame;
}
