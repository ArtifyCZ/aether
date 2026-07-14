# Aether

Aether is a hobby operating system kernel written in Rust and C, targeting
**x86_64** and **aarch64** architectures. It is a microkernel-inspired design
where a small, privileged kernel manages memory, scheduling, and hardware
interrupts, while the first userspace process (`init`) is responsible for
loading and launching further programs.

The project is in early stages of development. See [BACKLOG.md](BACKLOG.md)
for known limitations and planned improvements.

## Features

- Dual-architecture support: **x86_64** and **aarch64**
- Limine bootloader integration (UEFI + BIOS on x86_64, UEFI on aarch64)
- Pre-emptive cooperative scheduler with a null/idle task
- Hardware interrupt routing via APIC (x86_64) and GIC (aarch64)
- Memory-mapped I/O device mapping and virtual memory management
- ELF userspace process loading from an in-memory tarball (`initrd`)
- Syscall ABI defined in TOML and auto-generated for C and Rust userspace
- A Rust userspace runtime library (`aether_rt`) and syscall library (`aether_sys`)
- A C syscall stub library (`libsyscall`) for the `init` program
- Serial console output from both kernel and userspace
- Framebuffer terminal support

## Repository Layout

| Directory | Description |
|---|---|
| `kernel/entry` | Kernel entry point; uses the Limine bootloader protocol to set up allocators and call into `kernel_core` |
| `kernel/core` | Main kernel logic: scheduler, syscall dispatch, ELF loading, memory management, task registry |
| `kernel/hal` | Hardware Abstraction Layer — architecture-specific implementations (interrupts, MMU, tasks, timer, syscalls) |
| `kernel/platform` | Low-level C platform code used by `kernel_hal` (APIC, GIC, GDT, MMU page tables, etc.) |
| `kernel/api` | Shared ABI: `boot_info.h` passed to `init`, and the TOML-based syscall definition + code generators |
| `init` | First userspace process (C): parses the initrd, loads ELF binaries, exposes keyboard/serial drivers |
| `hello_world` | Minimal Rust userspace binary used for integration testing |
| `libs/aether_rt` | Rust userspace runtime (entry point, heap allocator, panic handler, libc shims) |
| `libs/aether_sys` | Auto-generated Rust syscall wrappers |
| `libs/libsyscall` | Auto-generated C syscall stubs and constants |
| `image` | Bazel rules to assemble the bootable disk image (ISO / raw) and initrd tarball |
| `tooling/qemu` | Script + Bazel rules to launch QEMU for either architecture |
| `tooling/sync_ide` | Scripts to regenerate `compile_commands.json` and `rust-project.json` for IDE tooling |
| `platforms` | Bazel platform definitions for cross-compilation |
| `toolchain` | Custom LLVM-based C/C++ cross-compilation toolchain configuration |
| `3rdparty` | Vendored Rust crates and external dependency BUILD files |
| `docs` | Extended documentation (architecture, building, syscalls, …) |

## Quick Start

### Prerequisites

- [Bazel](https://bazel.build/) (Bazelisk recommended)
- [QEMU](https://www.qemu.org/) — `qemu-system-x86_64` and/or `qemu-system-aarch64`
- For aarch64: QEMU with EDK2 firmware (provided automatically by Bazel via the `@qemu` external repository)

### Building and Running

Build and run in QEMU for **x86_64**:

```sh
bazel run //:qemu --config x86_64
```

Build and run in QEMU for **aarch64**:

```sh
bazel run //:qemu --config aarch64
```

Build only (without running):

```sh
bazel build //... --config x86_64
# or
bazel build //... --config aarch64
```

### Debugging

Start QEMU with a GDB/LLDB stub on port `1234` (execution paused at start):

```sh
bazel run //:qemu-debug --config x86_64
# or
bazel run //tooling/qemu:debug --config aarch64 -c dbg
```

Then attach your debugger. VS Code users can use the **"Debug kernel in Qemu"**
launch configuration in `.vscode/launch.json`. Zed users can use the
**"Qemu debug"** task in `.zed/debug.json`.

## Documentation

| Document | Contents |
|---|---|
| [docs/architecture.md](docs/architecture.md) | Component architecture, boot flow, and design decisions |
| [docs/building.md](docs/building.md) | Full build instructions and host prerequisites |
| [docs/development.md](docs/development.md) | IDE setup, debugging workflow, and Bazel tips |
| [docs/syscalls.md](docs/syscalls.md) | Syscall ABI reference and how to add new syscalls |
| [docs/hardware-interrupts.md](docs/hardware-interrupts.md) | IRQ conventions for x86_64 and aarch64 |
| [docs/kernel/init_contract.md](docs/kernel/init_contract.md) | Contract between the kernel and the `init` process |

## Licensing

The Aether project's licensing policy is located in the [LICENSE](LICENSE) file.

## Contributing

Contributions are welcome!

For more information on contributing to this project, please see [CONTRIBUTING.md](CONTRIBUTING.md).
