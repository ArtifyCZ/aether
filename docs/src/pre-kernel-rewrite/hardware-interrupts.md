# Hardware Interrupts

## IRQ

Interrupt Requests (IRQs) are used to identify hardware events.
They are represented as `uint8_t` values.
Userspace programs (such as a driver) can use syscalls to wait for an IRQ.

Before waiting for an IRQ, a userspace task must unmask it by calling
`irq_unmask(irq)`. Afterwards, `irq_wait(irq)` blocks the task until the
hardware raises the interrupt. See [syscalls.md](syscalls.md) for the full
syscall reference.

## x86_64

Since APIC is exclusively in use (meaning no legacy PIC),
there have been set some conventions for IRQ numbers.
For each IOAPIC pin, there is a corresponding IRQ number.
The pin's number is **always** identical to the IRQ number.

The interrupt vectors in range `0x00`-`0x1F` are used for CPU exceptions.
Due to legacy PIC and its conventions, `0x20`-`0x2F` are not used,
as it could lead to some undesired behavior.
Therefore, `0x30` has been chosen as the vector offset for the IRQs.
That means for every IRQ, its interrupt vector is `0x30` + the IRQ number.

## aarch64

On aarch64 (that is, 64-bit ARM architecture),
IRQs are used to represent shared peripheral interrupts (SPIs).
It has been decided that the IRQ numbers correspond to the SPI numbers.
Because interrupt ids (INTIDs) of SPIs start from `0x20`,
as the range `0x00`-`0x0F` is used for software generated interrupts (SGIs),
and `0x10`-`0x1F` is used for private peripheral interrupts (PPIs),
the IRQ number of an SPI is `0x20 + INTID`.
