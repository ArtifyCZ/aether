# Aether Architecture

## Overview

Aether is a microkernel-inspired operating system kernel targeting x86_64 and
aarch64. The kernel itself is small: it manages physical and virtual memory,
provides a preemptive scheduler, routes hardware interrupts to userspace tasks,
and exposes a minimal syscall interface. Everything else — device drivers,
program loading, IPC — lives in userspace.

## Boot Flow

1. The **Limine bootloader** loads the kernel ELF image and an `initrd` (tar
   archive) as a module. It also provides a framebuffer, memory map, RSDP, and
   command-line string to the kernel via the Limine protocol.
2. **`kernel/entry`** — the Rust entry point (`start.rs`) is called by Limine.
   It sets up a two-phase allocator (early bump allocator → paged allocator) and
   calls `kernel_core::main(...)`.
3. **`kernel/core` `main()`** initializes subsystems in order:
   - Virtual address allocator
   - Physical memory manager
   - Virtual memory manager (HHDM-aware)
   - Early serial console
   - Platform-specific initialization (ACPI parsing, APIC/GIC setup)
   - Framebuffer terminal
   - Interrupt controller (APIC on x86_64, GIC on aarch64)
   - Paged memory allocator (replaces the early bump allocator)
   - Task registry and scheduler
   - Syscall dispatcher
   - Timer (preemption tick at 100 Hz)
4. The kernel loads the **`init`** ELF binary from the initrd using its own ELF
   loader, creates a new address space, maps `boot_info` and the initrd into it,
   and spawns the `init` task.
5. The kernel disables its early console and switches to the scheduler. From
   this point on the kernel runs only in response to interrupts and syscalls.

## Component Map

```
┌────────────────────────────────────────────────────────────────────┐
│                         Bazel workspace                             │
│                                                                      │
│  kernel/entry ──► kernel/core ──► kernel/hal ──► kernel/platform   │
│                        │                                             │
│                        └──► kernel/api  (syscall ABI + boot_info)  │
│                                                                      │
│  init  ──► libs/libsyscall (C stubs)                                │
│         ──► kernel/api/init/boot_info.h                             │
│                                                                      │
│  hello_world ──► libs/aether_rt (Rust runtime)                      │
│               ──► libs/aether_sys (Rust syscall wrappers)           │
│                                                                      │
│  image/ ──► initrd (tar) + ISO/raw disk image                       │
└────────────────────────────────────────────────────────────────────┘
```

### `kernel/entry`

The bootloader-facing crate (`#![no_std]`, `#![no_main]`). Registers Limine
requests (framebuffer, memory map, HHDM offset, modules, RSDP, command line)
and implements the two-phase allocator hand-off:

- **Early allocator** — a simple bump allocator backed by a static buffer, used
  before page-frame allocation is available.
- **Proxy allocator** — a `#[global_allocator]` wrapper that delegates to
  whichever allocator is currently active and supports an atomic hand-off.

### `kernel/core`

The heart of the kernel. Key modules:

| Module | Responsibility |
|---|---|
| `scheduler` | Round-robin preemptive scheduler; supports null/idle task, IRQ wake-up, task exit |
| `syscall_handler` | Dispatches syscalls to per-syscall handlers |
| `task_registry` | Stores and manages all tasks (kernel threads and userspace processes) |
| `allocator` | Paged kernel heap (allocation only; deallocation not yet implemented) |
| `elf` | Loads ELF64 binaries into a new address space |
| `init_process` | Reads the initrd, finds the `init` binary, and spawns the init process |
| `tarball_parsing` | Minimal ustar tarball parser used to locate files in the initrd |
| `ticker` | Configures the hardware timer to fire at a fixed Hz and drive the scheduler |
| `platform/*` | Trait-based abstraction layer consumed by `kernel_core` |

The `platform/` sub-module contains Rust traits (e.g. `EarlyConsole`,
`PhysicalMemoryManager`, `Interrupts`, `Syscalls`, `VirtualMemoryManager`)
whose implementations live in `kernel/hal`.

### `kernel/hal`

The Hardware Abstraction Layer. Contains two sub-modules — `arch/x86_64` and
`arch/aarch64` — selected at compile time via `cfg(target_arch = …)`.

Each implementation covers:

- **CPU** — core initialization, per-CPU data structures, halting
- **Early console** — UART-based serial output used before the terminal is up
- **Emergency console** — fallback output used during kernel panics
- **Interrupts** — IDT/GDT (x86_64), exception vectors (aarch64), IRQ masking
- **MMU** — page table management, address space switching
- **Syscalls** — `syscall`/`sysret` (x86_64), `svc` (aarch64) entry/exit
- **Tasks** — task frame layout, context switching
- **Timer** — LAPIC timer (x86_64), ARM Generic Timer (aarch64)

### `kernel/platform`

Low-level C and assembly code that `kernel_hal` calls into. Includes:

- ACPI table parsing and APIC/IOAPIC/GIC discovery
- Page table structures for both architectures
- Physical memory map processing
- String and memory utilities

### `kernel/api`

Shared ABI between the kernel and userspace.

- **`init/boot_info.h`** — the `boot_info` struct passed by the kernel to the
  `init` process at startup (initrd pointer and size).
- **`syscalls/syscalls.toml`** — machine-readable syscall definitions (number,
  return type, argument names and types).
- **`syscalls/errors.toml`** — error codes returned by syscalls.
- **`syscalls/syscall_parser.py`** — shared Python library that parses the TOML
  definitions and drives code generators.
- **`syscalls/syscall_kernel_gen.py`** (in `kernel/syscalls/`) — generates the
  kernel-side dispatch glue.
- **`libs/libsyscall/syscall_c_stubs.py`** — generates `syscalls.h` for C
  userspace.
- **`libs/aether_sys/syscall_aether_sys_gen.py`** — generates the Rust
  `aether_sys` crate.

### `init`

The first userspace process (written in C). Its responsibilities are:

1. Initialize the serial port and use it as stdout.
2. Parse the initrd tarball and locate the `bin/hello_world` ELF binary.
3. Load the ELF binary into a new address space via `proc_create` / `proc_mmap`
   / `proc_mprot` syscalls.
4. Spawn the loaded process using `proc_spawn`.
5. Optionally handle keyboard input (driver in `src/drivers/keyboard/`).

### `hello_world`

A minimal Rust binary that prints a message via the `write` syscall and loops.
It exercises the `aether_rt` runtime and `aether_sys` syscall library.

### `libs/aether_rt`

The Rust userspace runtime crate. Provides:

- `_start` entry point — sets up the heap and calls `main`
- A heap allocator (linked-list allocator over a static buffer)
- Panic handler
- `libc`-style shims (`memcpy`, `memmove`, `memset`, `memcmp`, `__errno_location`)
- `__rust_probestack` / `_Unwind_Resume` stubs required by the Rust compiler

### `libs/aether_sys`

Auto-generated Rust bindings for every syscall and constant defined in
`syscalls.toml` / `errors.toml`. Regenerated by `bazel build //libs/aether_sys`.

### `libs/libsyscall`

Auto-generated C header (`syscalls.h`) and stubs for every syscall. Used by the
`init` program.

## Memory Layout

### Kernel

The kernel is loaded by Limine into the higher half of the virtual address space.
Limine provides a Higher-Half Direct Map (HHDM) offset so the kernel can access
all physical memory through a fixed virtual offset.

### Userspace

The lower half of the virtual address space is available to userspace processes.
Kernel-provided data (boot info, initrd, bootstrap stack) is mapped at the high
end of the lower half, growing downward. See
[kernel/init_contract.md](kernel/init_contract.md) for the exact layout
convention used for the `init` process.

## Syscall ABI

All syscalls follow a consistent convention:

- Each syscall has a unique number (0–255).
- The kernel returns an error code in one register and an optional return value
  in another.
- Using the return value when the error code is non-zero is undefined behavior.
- Up to 5 arguments are supported.

See [syscalls.md](syscalls.md) for the full reference.
