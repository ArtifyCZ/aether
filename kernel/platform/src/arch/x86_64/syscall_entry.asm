[bits 64]
extern syscalls_inner_handler
global syscalls_raw_handler

%define KERNEL_CS 0x08
%define KERNEL_SS 0x10
%define USER_SS   0x23
%define USER_CS   0x2B

syscalls_raw_handler:
    ; In x86-64, kernel addresses have the top bit set (Canonical High)
    test rcx, rcx
    js .from_kernel

.from_user:
    swapgs
    mov [gs:8], rsp     ; Save user RSP
    mov rsp, [gs:0]     ; Load kernel stack

    ; Construct the interrupt_frame
    push USER_SS        ; SS
    push qword [gs:8]   ; RSP (User stack)
    push r11            ; RFLAGS
    push USER_CS        ; CS
    jmp .continue_entry

.from_kernel:
    ; We are already in Ring 0. Do NOT swapgs.
    ; Do NOT switch stacks (RSP is already correct).
    push KERNEL_SS      ; SS
    push rsp            ; RSP (Note: this is the RSP *before* these pushes)
    ; Adjust the RSP we just pushed to account for the fact that
    ; it should point to the state BEFORE we started pushing the frame.
    add qword [rsp], 8

    push r11            ; RFLAGS
    push KERNEL_CS      ; CS
    ; Fall through

.continue_entry:
    push rcx            ; RIP
    push 0              ; Error code
    push 0x80           ; Dummy interrupt vector

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

    mov rax, cr3
    push rax            ; Save CR3 at [RSP]

    mov rdi, rsp
    call syscalls_inner_handler

    cli
    mov r12, rax        ; R12 = New frame pointer returned by scheduler

    ; 1. Restore CR3 safely
    mov rax, [r12]      ; Frame offset 0 is CR3
    mov cr3, rax

    ; 2. Point RSP to the start of the GPRs (skipping CR3)
    lea rsp, [r12 + 8]

    ; Check if we are returning to User Mode (RPL 3) or Kernel (RPL 0)
    ; CS is at [rsp + 0x90] (144 bytes from R15)
    mov rbx, [rsp + 0x90]
    test bl, 3
    jz .to_kernel

.to_user:
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
    pop rax             ; RAX is now the syscall return value

    add rsp, 16         ; Skip Error Code and Vector

    pop rcx             ; RIP for sysret
    add rsp, 8          ; Skip CS
    pop r11             ; RFLAGS for sysret
    pop r8              ; User RSP

    mov bx, 0x23
    mov ds, bx
    mov es, bx

    swapgs
    mov rsp, r8         ; Explicitly switch to user stack
    o64 sysret

.to_kernel:
    ; We are returning to Ring 0.
    ; We are ALREADY on the correct kernel stack (the one returned by the scheduler).
    ; We do NOT want to switch to the "User RSP" stored in the frame.

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

    ; Skip Error, Vector, RIP, CS, RFLAGS, RSP, SS
    ; Actually, we just need to use IRETQ to restore the execution state
    ; without moving to the 'User RSP' stored in the frame.
    add rsp, 16         ; Skip Error and Vector
    iretq               ; This pops RIP, CS, RFLAGS, RSP, SS
                        ; (Note: For Ring 0 -> Ring 0, these must be valid Kernel values)
