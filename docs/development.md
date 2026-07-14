# Development Guide

## IDE Support

Aether supports two language servers:

- **Rust** — Two Cargo workspaces (`/` and `/kernel/`) consumed by `rust-analyzer`.
- **C/C++** — `compile_commands.json` consumed by `clangd`.

### VS Code

The repository includes `.vscode/` configuration:

- **Recommended extensions** — see `.vscode/extensions.json`. Install them via
  _Extensions → … → Show Recommended Extensions_. The key extensions are
  `rust-lang.rust-analyzer` (Rust) and `llvm-vs-code-extensions.vscode-clangd`
  (C/C++).
- **Settings** (`.vscode/settings.json`) — pre-configures `rust-analyzer` to
  load Cargo workspaces, and `clangd` with the correct compile-commands directory.
- **Tasks** — the default build task (_Terminal → Run Build Task_, `Ctrl+Shift+B`)
  runs the `Sync Project (C + Rust)` command and prompts you to pick an
  architecture.

### Zed

The repository includes `.zed/` configuration:

- **Tasks** (`.zed/tasks.json`) — preconfigured tasks:
  - _Sync Project for x86_64 (C + Rust)_ — regenerates project files
  - _Qemu run x86_64_ / _Qemu run aarch64_ — run in QEMU
  - _Qemu debug aarch64_ — start QEMU with debugger stub
- **Debug** (`.zed/debug.json`) — a _Qemu debug_ configuration using `CodeLLDB`
  to attach to `127.0.0.1:1234`.
