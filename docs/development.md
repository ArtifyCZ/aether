# Development Guide

## IDE Support

Aether provides generated project files for two language servers:

- **Rust** — `rust-project.json` consumed by `rust-analyzer`.
- **C/C++** — `compile_commands.json` consumed by `clangd`.

It is not necessary to sync before opening the project for the first time; the
files may already be present in the repository. Run the sync command to pick up
changes made since the last sync (new or removed source files, modified
`BUILD.bazel` targets, or a different target architecture), then restart the
affected language server (`rust-analyzer`, `clangd`, or both):

```sh
# x86_64 targets
bazel run //tooling/sync_ide --config x86_64

# aarch64 targets
bazel run //tooling/sync_ide --config aarch64
```

This invokes two helper scripts:
- `discover_bazel_c_compile_commands.py` — uses `bazel aquery` to extract C
  compile commands and writes `compile_commands.json` to the workspace root.
- `discover_bazel_rust_project.sh` — uses `bazel build` with
  `@rules_rust//tools/rust_analyzer:gen_rust_project` to generate
  `rust-project.json` for `rust-analyzer`.

### VS Code

The repository includes `.vscode/` configuration:

- **Recommended extensions** — see `.vscode/extensions.json`. Install them via
  *Extensions → … → Show Recommended Extensions*. The key extensions are
  `rust-lang.rust-analyzer` (Rust) and `llvm-vs-code-extensions.vscode-clangd`
  (C/C++).
- **Settings** (`.vscode/settings.json`) — pre-configures `rust-analyzer` to
  load `rust-project.json` from the workspace root, and `clangd` with the
  correct compile-commands directory and query-driver path for the Bazel-managed
  LLVM toolchain.
- **Tasks** — the default build task (*Terminal → Run Build Task*, `Ctrl+Shift+B`)
  runs the `Sync Project (C + Rust)` command and prompts you to pick an
  architecture.
- **Launch configurations** (`.vscode/launch.json`) — three configurations for
  attaching `lldb-dap` to a running QEMU GDB stub:
  - *Debug kernel in Qemu* — attaches to `bazel-bin/kernel/entry/kernel`
  - *Debug init in Qemu* — attaches to `bazel-bin/init/init`
  - *Debug hello_world in Qemu* — attaches to `bazel-bin/hello_world/hello_world`

### Zed

The repository includes `.zed/` configuration:

- **Tasks** (`.zed/tasks.json`) — preconfigured tasks:
  - *Sync Project for x86_64 (C + Rust)* — regenerates project files
  - *Qemu run x86_64* / *Qemu run aarch64* — run in QEMU
  - *Qemu debug aarch64* — start QEMU with debugger stub
  - *Reformat Bazel* — run `buildifier` on all BUILD files
- **Debug** (`.zed/debug.json`) — a *Qemu debug* configuration using `CodeLLDB`
  to attach to `127.0.0.1:1234`.

## Debugging Workflow

### 1. Start QEMU with debugging enabled

```sh
# x86_64
bazel run //:qemu-debug --config x86_64

# aarch64 (use -c dbg for unoptimized build)
bazel run //tooling/qemu:debug --config aarch64 -c dbg
```

QEMU starts paused, waiting for a debugger on port `1234`.

### 2. Attach your debugger

**VS Code / lldb-dap:**
Select the appropriate configuration in the *Run and Debug* panel and press
*Start Debugging* (F5). Make sure QEMU is already running and paused.

**Zed / CodeLLDB:**
Run the *Qemu debug* task in the debug panel.

**Command-line LLDB:**
```sh
lldb bazel-bin/kernel/entry/kernel
(lldb) gdb-remote 1234
```

**Command-line GDB:**
```sh
gdb bazel-bin/kernel/entry/kernel
(gdb) target remote :1234
(gdb) continue
```

### Debugging userspace (init, hello_world)

To debug a userspace binary, attach with the path to that binary's ELF file
instead of the kernel. The symbols and source locations are read from there.
For example in VS Code, use the *Debug init in Qemu* or
*Debug hello_world in Qemu* launch configurations.

> **Tip:** Building with `-c dbg` (`bazel run ... -c dbg`) disables compiler
> optimizations and produces richer debug information.

## Adding a New Source File

Because Aether uses Bazel, new source files must be added to the relevant
`BUILD.bazel` target in addition to being created on disk. After doing so, run
the IDE sync command again to pick up the new file in `compile_commands.json` /
`rust-project.json`.

## Bazel Tips

- `bazel clean` — clean cached build outputs.
- `bazel clean --expunge` — deep clean, including external deps.
- `bazel query //...` — list available build targets.
- `bazel query 'deps(//kernel/entry:kernel)'` — show what a target depends on.
- `bazel build //... --config x86_64 --verbose_failures` — build with verbose output.
- `buildifier --mode=fix --lint=fix -r .` — format all BUILD files.
