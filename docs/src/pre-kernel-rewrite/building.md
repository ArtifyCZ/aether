# Building Aether

## Host Prerequisites

- **GNU Make** — Used for orchestrating builds. Doesn't have to be in `PATH`.
- **QEMU** — Required for running: `qemu-system-x86_64` (x86_64) and/or
  `qemu-system-aarch64` (aarch64).
- **Python 3** — Required by code-generation scripts.
- **Clang / LLVM** — Used for compiling C/ASM code, for linking, and binutils.
- **Rust** — Used for compiling Rust code.

## Architecture Configuration

All build and run commands accept an `ARCH` variable that selects the target
architecture:

- `ARCH=x86_64` — 64-bit x86
- `ARCH=aarch64` — 64-bit ARM

## Building

Build everything for x86_64:

```sh
make all ARCH=x86_64
```

Build everything for aarch64:

```sh
make all ARCH=aarch64
```

You can also build individual targets, for example:

```sh
# Build only the kernel ELF
make pkg/kernel/install ARCH=x86_64

# Build the init binary
make pkg/init/install ARCH=x86_64

# Build the hello_world binary
make pkg/hello_world/install ARCH=x86_64

# Build the bootable ISO image
make dist/aether-x86_64.iso ARCH=x86_64
# or
make dist/aether-aarch64.img ARCH=aarch64
```
