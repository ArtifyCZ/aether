# Building Aether

## Host Prerequisites

- **Bazel** — The project uses [Bazelisk](https://github.com/bazelbuild/bazelisk);
  place it on your `PATH` as `bazel`.
- **QEMU** — Required for running: `qemu-system-x86_64` (x86_64) and/or
  `qemu-system-aarch64` (aarch64).
- **Python 3** — Required by Bazel for code-generation scripts.
- **Clang / LLVM** — Automatically downloaded by Bazel via `toolchains_llvm`
  (LLVM 19.1.1). No manual install needed.
- **Rust nightly** — Automatically downloaded by Bazel via `rules_rust`
  (nightly 2026-03-26). No manual install needed.

> **Note:** The toolchain (LLVM, Rust) is managed entirely by Bazel. You do not
> need a system-wide Clang or Rust installation — Bazel will fetch the correct
> versions the first time you run a build.

## Architecture Configuration

All build and run commands accept a `--config` flag that selects the target
architecture:

- `--config x86_64` — 64-bit x86
- `--config aarch64` — 64-bit ARM

## Building

Build everything for x86_64:

```sh
bazel build //... --config x86_64
```

Build everything for aarch64:

```sh
bazel build //... --config aarch64
```

You can also build individual targets, for example:

```sh
# Build only the kernel ELF
bazel build //kernel/entry:kernel --config x86_64

# Build the init binary
bazel build //init --config x86_64

# Build the hello_world binary
bazel build //hello_world --config x86_64

# Build the bootable ISO image
bazel build //image/iso --config x86_64
```

### Build outputs

After a successful build, Bazel symlinks output into `bazel-bin/`:

- `bazel-bin/kernel/entry/kernel` — Kernel ELF binary
- `bazel-bin/init/init` — Init process ELF binary
- `bazel-bin/hello_world/hello_world` — hello_world ELF binary
- `bazel-bin/image/iso/` — Bootable ISO (x86_64)
- `bazel-bin/image/img/` — Bootable raw disk image (aarch64)
- `bazel-bin/image/initrd/initrd.tar` — initrd tarball

## Running in QEMU

These commands build all dependencies and launch QEMU in one step:

```sh
# x86_64
bazel run //:qemu --config x86_64

# aarch64
bazel run //:qemu --config aarch64
```

QEMU is started with 2 GB RAM, no reboot on crash, and serial output forwarded
to `stdio`.

## Debugging

Start QEMU with GDB/LLDB remote debugging enabled (execution paused at entry):

```sh
# x86_64 (normal optimizations)
bazel run //:qemu-debug --config x86_64

# aarch64 (debug build, -c dbg disables optimizations)
bazel run //tooling/qemu:debug --config aarch64 -c dbg
```

QEMU will listen on `127.0.0.1:1234`. Attach with:

```sh
# GDB (example)
gdb bazel-bin/kernel/entry/kernel
(gdb) target remote :1234

# LLDB (example)
lldb bazel-bin/kernel/entry/kernel
(lldb) gdb-remote 1234
```

See [development.md](development.md) for IDE-integrated debugging setup.

## Formatting Bazel Files

If you modify `BUILD.bazel` or `MODULE.bazel` files, reformat them with
[`buildifier`](https://github.com/bazelbuild/buildtools):

```sh
buildifier --mode=fix --lint=fix -r .
```
