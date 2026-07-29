# Aether

Aether is a hobby operating system in development.
Code is written primarily in the [Rust](https://rust-lang.org/) language.
The OS targets rather modern machines with x86_64 (amd64) and aarch64 (64-bit arm) architectures.

The architecture is of a [microkernel](https://en.wikipedia.org/wiki/Microkernel) design.
Most OS responsibilities should be fulfilled by userspace servers,
as opposed to fulfilled by a kernel of [monolithic](https://en.wikipedia.org/wiki/Monolithic_kernel) design.

The project is in its early stages of development.
See [BACKLOG.md](BACKLOG.md) for known limitations and planned improvements.

> Note that currently the [kernel](kernel) is being rewritten.

## Features

- Dual-architecture support: **x86_64** and **aarch64**
- Limine bootloader integration (UEFI + BIOS on x86_64, UEFI on aarch64)
- Framebuffer early state logging support

## Quick Start

### Documentation

The documentation is available in the [docs/ directory](docs/README.md).

Note that due to the ongoing kernel rewrite,
the docs are not up to date right now.

### Prerequisites

- [GNU Make](https://www.gnu.org/software/make/)
- [QEMU](https://www.qemu.org/) — `qemu-system-x86_64` and/or `qemu-system-aarch64`
- For aarch64: QEMU with EDK2 firmware (provided in the `3rdparty/edk2/` directory)

### Building and Running

Build and run in QEMU:

```sh
make qemu ARCH=x86_64
# or
make qemu ARCH=aarch64
```

Build only (without running):

```sh
make all ARCH=x86_64
# or
make all ARCH=aarch64
```

### Debugging

Start QEMU with a GDB/LLDB stub on port `1234` (execution paused at start):

```sh
make qemu ARCH=x86_64 DEBUG=1
# or
make qemu ARCH=aarch64 DEBUG=1
```

Then attach your debugger.

## Licensing

The Aether project's licensing policy is located in the [LICENSE](LICENSE) file.

## Contributing

Contributions are welcome!

For more information on contributing to this project, please see [CONTRIBUTING.md](CONTRIBUTING.md).
