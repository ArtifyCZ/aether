# Backlog

## Ideas

- Don't allocate a kernel stack for every single task, but just for the kernel
threads, use ist for interrupts, and store for every task its current context
(registers). Could also use a single stack for interrupts and syscalls per CPU.
- Remove support for the use of `int 0x80` to invoke a syscall.
