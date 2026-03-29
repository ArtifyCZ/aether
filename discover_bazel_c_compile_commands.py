import argparse
import json
import os
import subprocess
import sys

# 1. Parse Args to allow dynamic config switching
parser = argparse.ArgumentParser()
parser.add_argument("--config", default="x86_64", help="Bazel config/platform")
args_cli = parser.parse_args()

info = {
    k: subprocess.check_output(["bazel", "info", k]).decode().strip()
    for k in ["execution_root", "output_base", "workspace"]
}

print(f"Extracting Bazel truth for {args_cli.config}...")
aquery_kernel_cmd = [
    "bazel",
    "aquery",
    "kind(cc_.*, //kernel/...)",
    f"--config={args_cli.config}",
    "--output=jsonproto",
]
kernel_data = json.loads(subprocess.check_output(aquery_kernel_cmd))
aquery_userspace_cmd = [
    "bazel",
    "aquery",
    "kind(cc_.*, //...) - //kernel/...",
    f"--config={args_cli.config}",
    "--output=jsonproto",
]
userspace_data = json.loads(subprocess.check_output(aquery_userspace_cmd))
actions = kernel_data.get("actions", []) + userspace_data.get("actions", [])

commands = []
for action in actions:
    if "arguments" in action:
        args = list(action["arguments"])  # Copy the list

        # 1. Fix include paths in the arguments
        new_args = []
        i = 0
        while i < len(args):
            arg = args[i]

            # 1. Handle space-separated flags: -I path, -iquote path, -isystem path
            if arg in ["-I", "-isystem", "-iquote"] and i + 1 < len(args):
                path = args[i + 1]
                abs_path = (
                    os.path.join(info["execution_root"], path)
                    if not os.path.isabs(path)
                    else path
                )
                new_args.extend([arg, abs_path])
                i += 2
                continue

            # 2. Handle mashed flags: -Ipath, -iquotepath
            # Note: -isystem usually isn't mashed, but we handle it just in case
            found_mashed = False
            for prefix in ["-I", "-iquote", "-isystem"]:
                if arg.startswith(prefix) and len(arg) > len(prefix):
                    path = arg[len(prefix) :]
                    abs_path = (
                        os.path.join(info["execution_root"], path)
                        if not os.path.isabs(path)
                        else path
                    )
                    new_args.append(f"{prefix}{abs_path}")
                    found_mashed = True
                    break

            if found_mashed:
                i += 1
                continue

            # 3. Handle everything else (including the source file and output paths)
            new_args.append(arg)
            i += 1

        if new_args and "clang_wrapper.sh" in new_args[0]:
            # You can usually just replace it with 'clang'
            # as long as the rest of the flags are standard.
            new_args[0] = "clang"

        # 2. Find the file for the command
        src_rel = next((a for a in new_args if a.endswith((".c", ".cpp", ".h"))), None)

        if src_rel:
            # Use absolute path for the file key so clangd knows exactly which buffer it is
            if not os.path.isabs(src_rel):
                local_path = os.path.abspath(os.path.join(info["workspace"], src_rel))
                final_file_path = (
                    local_path
                    if os.path.exists(local_path)
                    else os.path.join(info["execution_root"], src_rel)
                )
            else:
                final_file_path = src_rel

            commands.append(
                {
                    "directory": info["workspace"],
                    "arguments": new_args,  # Use the patched arguments
                    "file": final_file_path,
                }
            )

with open("compile_commands.json.tmp", "w") as f:
    json.dump(commands, f, indent=2)
os.rename("compile_commands.json.tmp", "compile_commands.json")

print(f"Done. Synchronized {len(commands)} commands for {args_cli.config}.")

# We use 'output_base' for 'external' because execution_root/external
# is often just a collection of fragile symlinks.
for link, target_key in [("external", "output_base"), ("bazel-out", "execution_root")]:
    target = os.path.join(info[target_key], link)
    if os.path.lexists(link):
        os.remove(link)
    os.symlink(target, link)

print(f"Done. Synchronized {len(commands)} commands.")
