# Backlog

## Ideas

- Add lint/warning ignores for unused type aliases and stuff in the generated bindings.
- Don't allocate a kernel stack for every single task, but just for the kernel
threads, use ist for interrupts, and store for every task its current context
(registers). Could also use a single stack for interrupts and syscalls per CPU.

## Bugs

- There might be a bug in the handling of the `syscall` instruction, particularly when
switching to a different user-space task, as the rcx register is getting overwritten.
