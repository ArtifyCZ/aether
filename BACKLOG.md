# Backlog

## Ideas

- Add lint/warning ignores for unused type aliases and stuff in the generated bindings.
- Don't allocate a kernel stack for every single task, but just for the kernel
threads, use ist for interrupts, and store for every task its current context
(registers). Could also use a single stack for interrupts and syscalls per CPU.
- Move the storing of the current task's id from the scheduler to GS base. By that is meant
to move it to the CPU-local storage and have some interface for scheduler to retrieve the current
task's id and to set it. The scheduler also could get the current task's id provided as a parameter.
The current task's id stored in GS base could be set as a part of the task_prepare_switch function.

## Bugs

- There might be a bug in the handling of the `syscall` instruction, particularly when
switching to a different user-space task, as the rcx register is getting overwritten.
This might be solvable by switching to the `iretq` instruction when returning to a different
task than the one that invoked the syscall. This way is already used when returning from a syscall
to a kernel thread.
