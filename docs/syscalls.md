# Syscall ABI Reference

## Overview

Syscalls are the interface between userspace processes and the kernel. The
Aether syscall ABI is defined in TOML files under `kernel/api/syscalls/`:

- **`syscalls.toml`** — the current list of all defined syscalls.
- **`errors.toml`** — all possible syscall error codes.

These files are used to automatically generate C headers and Rust bindings for
userspace consumers.

> The list of syscalls and their functionality will change as the project
> evolves. Refer to `syscalls.toml` and `errors.toml` for the current
> authoritative definition rather than any derived documentation.

## Calling Convention

### Syscall number and arguments

The syscall number and arguments are passed in CPU registers:

**x86_64**

- Syscall number: `rax`
- Arguments (in order): `rdi`, `rsi`, `rdx`, `r10`, `r8`

**aarch64**

- Syscall number: `x8`
- Arguments (in order): `x0`, `x1`, `x2`, `x3`, `x4`

Up to **5 arguments** per syscall are currently supported. This limit may be
increased in the future.

### Return values

Every syscall returns two values:

- An **error code** — always present.
- A **return value** — present only if the syscall produces one.

An error code of `0` means success. **Using the return value when the error
code is non-zero is undefined behavior.**

All possible error codes for the current syscall set are defined in
`kernel/api/syscalls/errors.toml`.

## Adding a New Syscall

1. **Define the syscall** in `kernel/api/syscalls/syscalls.toml`:

   ```toml
   [syscalls.my_syscall]
   number = 0x0C          # unique number 0–255
   return_type = "uint64" # one of the valid types
   args = [
       { name = "foo", type = "uint32" },
   ]
   ```

2. **Implement the handler** in `kernel/core/src/syscall_handler/`. Create a
   new file `sys_my_syscall.rs` and implement the logic. Register the handler
   in `kernel/core/src/syscall_handler/mod.rs`.

3. **Regenerate bindings** by building the relevant targets:

   ```sh
   bazel build //libs/libsyscall //libs/aether_sys --config x86_64
   ```

   The generated C header is written to
   `bazel-bin/libs/libsyscall/syscalls.h` and the Rust crate sources to
   `bazel-bin/libs/aether_sys/`.

4. **Add error codes** (if needed) in `kernel/api/syscalls/errors.toml`.

