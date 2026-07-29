# Backlog

## Ideas

- Make the kernel provide only a single page-sized stack to the init program
  by convention. This initial stack is there just for the convenience,
  it would be technically possible for the init program to bootstrap its own stack.
  The init program then during its start/entrypoint function maps additional memory
  as its actual stack and switches to this new one (by writing to the rsp register).
  This whole stack idea is so that the init program doesn't have to guess the lowest
  address of the kernel-provided stack; and so that the kernel doesn't have to guess
  how much memory is required by the init program.

## Bugs
