.section .text
.global _start

_start:
    /* 1. Capture the Limine stack pointer (currently SP_EL0) */
    mov x0, sp

    /* 2. Force 16-byte alignment */
    bic x0, x0, #0xf

    /* 3. Load the magic value using ldr instead of mov */
    ldr x18, =0xDEADD00D

    /* 4. Switch to SP_EL1 */
    msr spsel, #1
    isb

    /* 5. Now that SPSel is 1, 'sp' is the alias for SP_EL1.
       We set it to our virtual address. */
    mov sp, x0
    isb

    /* 6. Jump to C */
    bl boot

1:  wfi
    b 1b
