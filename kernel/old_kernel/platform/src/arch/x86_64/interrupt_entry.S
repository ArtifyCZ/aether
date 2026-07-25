.extern x86_64_interrupt_dispatcher

/* Enable alternate macro mode for expansion */
.altmacro

.macro STUB_NO_ERROR num
.global interrupt_stub_\num
interrupt_stub_\num:
    pushq $0              /* Dummy error code */
    pushq $\num           /* Interrupt number */
    jmp common_stub
.endm

.macro STUB_WITH_ERROR num
.global interrupt_stub_\num
interrupt_stub_\num:
    /* CPU already pushed an error code here! */
    pushq $\num           /* Interrupt number */
    jmp common_stub
.endm

/* Helper macro to force evaluation of the index in the quad table */
.macro PUSH_STUB_ADDRESS index
    .quad interrupt_stub_\index
.endm

common_stub:
    /* Save all general purpose registers */
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rdi
    pushq %rsi
    pushq %rbp
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    /* Save the current pages table address */
    movq %cr3, %rax
    pushq %rax

    movq %rsp, %rdi        /* Pass the stack pointer as the interrupt_frame pointer */
    call x86_64_interrupt_dispatcher
    movq %rax, %rsp        /* Switch stack to the returned one from the dispatcher */

    /* Restore the pages table address */
    popq %rax
    movq %rax, %cr3

    /* Restore registers */
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rbp
    popq %rsi
    popq %rdi
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax

    addq $16, %rsp         /* Clean up interrupt_number and error_code */
    iretq

/* Hand-coded stubs for exceptions */
STUB_NO_ERROR 0
STUB_NO_ERROR 1
STUB_NO_ERROR 2
STUB_NO_ERROR 3
STUB_NO_ERROR 4
STUB_NO_ERROR 5
STUB_NO_ERROR 6
STUB_NO_ERROR 7

STUB_WITH_ERROR 8 /* Double Fault */

STUB_NO_ERROR 9

STUB_WITH_ERROR 10 /* Invalid TSS */
STUB_WITH_ERROR 11 /* Segment Not Present */
STUB_WITH_ERROR 12 /* Stack Segment Fault */
STUB_WITH_ERROR 13 /* General Protection Fault */
STUB_WITH_ERROR 14 /* Page Fault */

STUB_NO_ERROR 15
STUB_NO_ERROR 16

STUB_WITH_ERROR 17 /* Alignment Check */

STUB_NO_ERROR 18
STUB_NO_ERROR 19
STUB_NO_ERROR 20

STUB_WITH_ERROR 21 /* Control Protection Exception */

STUB_NO_ERROR 22
STUB_NO_ERROR 23
STUB_NO_ERROR 24
STUB_NO_ERROR 25
STUB_NO_ERROR 26
STUB_NO_ERROR 27
STUB_NO_ERROR 28

STUB_WITH_ERROR 29 /* VMM Communication Exception */
STUB_WITH_ERROR 30 /* Security Exception */

/* Generate stubs for 31 to 255 */
.set i, 31
.rept 225
    STUB_NO_ERROR %i
    .set i, i + 1
.endr

.global interrupt_stubs
interrupt_stubs:
.set j, 0
.rept 256
    PUSH_STUB_ADDRESS %j
    .set j, j + 1
.endr
