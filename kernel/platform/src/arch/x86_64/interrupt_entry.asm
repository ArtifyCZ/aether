[bits 64]
extern x86_64_interrupt_dispatcher

%macro STUB_NO_ERROR 1
global interrupt_stub_%1
interrupt_stub_%1:
    push 0              ; Dummy error code
    push %1             ; Interrupt number
    jmp common_stub
%endmacro

%macro STUB_WITH_ERROR 1
global interrupt_stub_%1
interrupt_stub_%1:
    ; CPU already pushed an error code here!
    push %1             ; Interrupt number
    jmp common_stub
%endmacro

common_stub:
    ; Save all general purpose registers
    push rax
    push rbx
    push rcx
    push rdx
    push rdi
    push rsi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    ; Save the current pages table address
    mov rax, cr3
    push rax

    mov rdi, rsp        ; Pass the stack pointer as the interrupt_frame pointer
    call x86_64_interrupt_dispatcher
    mov rsp, rax        ; Switch stack to the returned one from the dispatcher

    ; Restore the pages table address
    pop rax
    mov cr3, rax

    ; Restore registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rsi
    pop rdi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    add rsp, 16         ; Clean up interrupt_number and error_code
    iretq


STUB_NO_ERROR 0
STUB_NO_ERROR 1
STUB_NO_ERROR 2
STUB_NO_ERROR 3
STUB_NO_ERROR 4
STUB_NO_ERROR 5
STUB_NO_ERROR 6
STUB_NO_ERROR 7

STUB_WITH_ERROR 8 ; Double Fault

STUB_NO_ERROR 9

STUB_WITH_ERROR 10 ; Invalid TSS
STUB_WITH_ERROR 11 ; Segment Not Present
STUB_WITH_ERROR 12 ; Stack Segment Fault
STUB_WITH_ERROR 13 ; General Protection Fault
STUB_WITH_ERROR 14 ; Page Fault

STUB_NO_ERROR 15
STUB_NO_ERROR 16

STUB_WITH_ERROR 17 ; Alignment Check

STUB_NO_ERROR 18
STUB_NO_ERROR 19
STUB_NO_ERROR 20

STUB_WITH_ERROR 21 ; Control Protection Exception

STUB_NO_ERROR 22
STUB_NO_ERROR 23
STUB_NO_ERROR 24
STUB_NO_ERROR 25
STUB_NO_ERROR 26
STUB_NO_ERROR 27
STUB_NO_ERROR 28

STUB_WITH_ERROR 29 ; VMM Communication Exception
STUB_WITH_ERROR 30 ; Security Exception

; Generate stubs for 31 to 255
%assign i 31
%rep 225
    STUB_NO_ERROR i
%assign i i+1
%endrep


global interrupt_stubs
interrupt_stubs:
%assign i 0
%rep    256
    dq interrupt_stub_%+i
%assign i i+1
%endrep
