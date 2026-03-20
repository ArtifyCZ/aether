.extern syscalls_inner_handler
.global syscalls_raw_handler

.set KERNEL_CS, 0x08
.set KERNEL_SS, 0x10
.set USER_SS,   0x23
.set USER_CS,   0x2B

syscalls_raw_handler:
    /* In x86-64, kernel addresses have the top bit set (Canonical High) */
    testq %rcx, %rcx
    js .from_kernel

.from_user:
    swapgs
    movq %rsp, %gs:8     /* Save user RSP to absolute GS offset 8 */
    movq %gs:0, %rsp     /* Load kernel stack from absolute GS offset 0 */

    /* Force 16-byte alignment of the BASE */
    andq $-16, %rsp

    pushq $0 /* PADDING: Make the total pushes even (24) */

    /* Construct the interrupt_frame */
    pushq $USER_SS
    pushq %gs:8          /* RSP (User stack) */
    pushq %r11
    pushq $USER_CS
    jmp .continue_entry

.from_kernel:
    /* We are already in Ring 0. Do NOT swapgs. */
    /* Do NOT switch stacks (RSP is already correct). */
    pushq $KERNEL_SS      /* SS */
    pushq %rsp           /* RSP (Note: this is the RSP *before* these pushes) */

    /* Adjust the RSP we just pushed to account for the state BEFORE the frame */
    addq $8, (%rsp)

    pushq %r11           /* RFLAGS */
    pushq $KERNEL_CS      /* CS */
    /* Fall through */

.continue_entry:
    pushq %rcx           /* RIP */
    pushq $0             /* Error code */
    pushq $0x80          /* Dummy interrupt vector */

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

    movq %cr3, %rax
    pushq %rax           /* Save CR3 at [RSP] */

    movq %rsp, %rdi
    call syscalls_inner_handler

    cli
    movq %rax, %r12      /* R12 = New frame pointer returned by scheduler */

    /* 1. Restore CR3 safely */
    movq (%r12), %rax    /* Frame offset 0 is CR3 */
    movq %rax, %cr3

    /* 2. Point RSP to the start of the GPRs (skipping CR3) */
    leaq 8(%r12), %rsp

    /* Check if we are returning to User Mode (RPL 3) or Kernel (RPL 0) */
    /* CS is at [rsp + 0x90] (144 bytes from R15) */
    movq 0x90(%rsp), %rbx
    testb $3, %bl
    jz .to_kernel

.to_user:
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
    popq %rax            /* RAX is now the syscall return value */

    addq $16, %rsp        /* Skip Error Code and Vector */

    popq %rcx            /* RIP for sysret */
    addq $8, %rsp         /* Skip CS */
    popq %r11            /* RFLAGS for sysret */
    popq %r8             /* User RSP */

    movw $0x23, %bx
    movw %bx, %ds
    movw %bx, %es

    swapgs
    movq %r8, %rsp
    sysretq              /* sysretq implies 64-bit return in GAS; no prefix needed */

.to_kernel:
    /* Returning to Ring 0. ALREADY on correct kernel stack. */
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

    addq $16, %rsp        /* Skip Error and Vector */
    iretq                /* Pops RIP, CS, RFLAGS, RSP, SS */
