import json
import subprocess
import os
import sys
import argparse

# 1. Parse Args to allow dynamic config switching
parser = argparse.ArgumentParser()
parser.add_argument("--config", default="x86_64", help="Bazel config/platform")
args_cli = parser.parse_args()

info = {k: subprocess.check_output(["bazel", "info", k]).decode().strip() 
        for k in ["execution_root", "output_base", "workspace"]}

print(f"Extracting Bazel truth for {args_cli.config}...")
aquery_cmd = [
    "bazel", "aquery", "kind(cc_.*, //kernel/...)",
    f"--config={args_cli.config}", "--output=jsonproto"
]
data = json.loads(subprocess.check_output(aquery_cmd))

commands = []
for action in data.get("actions", []):
    if "arguments" in action:
        args = action["arguments"]
        if any(x in args[0] for x in ["clang", "gcc"]):
            # Find the source file relative path from the arguments
            src_rel = next((a for a in args if a.endswith((".c", ".cpp", ".h"))), None)
            
            if src_rel:
                # TRUTH MAPPING:
                # If the file exists in our local workspace, use the local path.
                # This ensures clangd matches the file you actually have open.
                local_path = os.path.abspath(os.path.join(info["workspace"], src_rel))
                
                if os.path.exists(local_path):
                    final_file_path = local_path
                else:
                    # Otherwise, it's a generated file or external header
                    final_file_path = os.path.join(info["execution_root"], src_rel)

                commands.append({
                    "directory": info["workspace"], # Run clangd from workspace root
                    "arguments": args,
                    "file": final_file_path
                })

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
