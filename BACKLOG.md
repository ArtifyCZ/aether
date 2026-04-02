# Backlog

## Ideas

- Add lint/warning ignores for unused type aliases and stuff in the generated bindings.
- Don't allocate a kernel stack for every single task, but just for the kernel
threads, use ist for interrupts, and store for every task its current context
(registers). Could also use a single stack for interrupts and syscalls per CPU.
- Use two registers for syscall return values, one for the error code, the other for the result.
- Update the physical memory manager so that it can be used to allocate contiguous memory chunks.
- Add some structure for each address space to know what addresses are used and what are available.
It should also provide functions for allocating ranges of addresses.
- Make kernel provide only a single page-sized stack to the init program
by convention. This initial stack is there just for the convenience,
it would be technically possible for the init program to bootstrap its own stack.
The init program then during its start/entrypoint function maps additional memory
as its actual stack and switches to this new one (by writing to the rsp register).
This whole stack idea is so that the init program doesn't have to guess the lowest
address of the kernel-provided stack; and so that the kernel doesn't have to guess
how much memory is required by the init program.

## Bugs

- There might be a bug in the handling of the `syscall` instruction, particularly when
switching to a different user-space task, as the rcx register is getting overwritten.
This might be solvable by switching to the `iretq` instruction when returning to a different
task than the one that invoked the syscall. This way is already used when returning from a syscall
to a kernel thread.
- The kernel memory allocator supports only allocation, no freeing.
- VirtualMemoryManagerContext is leaking memory when it is destroyed, as the page table is not freed.
