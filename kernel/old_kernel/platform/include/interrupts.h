#pragma once

#include <stdint.h>
#include <stdbool.h>

// Represents the CPU state saved during an interrupt.
// This will be defined per-architecture in a separate header.
struct interrupt_frame;

// Callback signature for an interrupt handler.
// Returns true if the interrupt was handled.
typedef bool (*irq_handler_t)(struct interrupt_frame **frame, void *priv);
typedef bool (*irq_handler_new_t)(struct interrupt_frame **frame, uint8_t irq, void *priv);

// High-level interrupt types
typedef enum {
    IRQ_TYPE_EDGE_RISING,
    IRQ_TYPE_EDGE_FALLING,
    IRQ_TYPE_LEVEL_HIGH,
    IRQ_TYPE_LEVEL_LOW,
} irq_type_t;

/**
 * Global interrupt management
 */

// Initialize the architecture-specific interrupt controller (GIC or APIC)
// UNUSED OUTSIDE RUST
void interrupts_init(void);

// UNUSED OUTSIDE RUST
void interrupts_set_irq_handler(irq_handler_new_t handler, void *priv);

// UNUSED OUTSIDE RUST
void interrupts_mask_irq(uint8_t irq);

// UNUSED OUTSIDE RUST
void interrupts_unmask_irq(uint8_t irq);

/**
 * IRQ Routing
 */

// Registers a handler for a specific IRQ line
// irq: The hardware IRQ number
// handler: The function to call
// priv: Private data passed back to the handler
// USED TO REGISTER LAPIC HANDLER
bool interrupts_register_handler(uint32_t irq, irq_handler_t handler, void *priv);

// Unregister a handler
// UNUSED
bool interrupts_unregister_handler(uint32_t irq);

// Configure an IRQ (trigger type, priority, etc.)
// UNUSED
void interrupts_configure_irq(uint32_t irq, irq_type_t type, uint8_t priority);
