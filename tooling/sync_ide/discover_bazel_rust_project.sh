#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$BUILD_WORKSPACE_DIRECTORY"
output_path="${workspace_dir}/rust-project.json"

cd "${workspace_dir}"

ARCH="x86_64"
[[ $1 != "" ]] && ARCH="$1"

tmp_output="$(mktemp "${TMPDIR:-/tmp}/rust-project.raw.XXXXXX")"
tmp_json="$(mktemp "${TMPDIR:-/tmp}/rust-project.json.XXXXXX")"

cleanup() { rm -f "${tmp_output}" "${tmp_json}"; }
trap cleanup EXIT

# FORCE GENERATION: Actually build the files so rust-analyzer can read them
# That is necessary for the generated files, such as C bindings and syscall stubs, to be accessible by rust-analyzer
echo "Building generated artifacts for $ARCH to ensure they exist on disk..."
bazel build //libs/aether_sys //kernel/api/init:init_contract_rust --platforms=//platforms:user_"$ARCH" --noshow_progress
bazel build //kernel/syscalls //kernel/api/init:init_contract_rust //kernel/core:kernel_bindings_gen --platforms=//platforms:kernel_"$ARCH" --noshow_progress

bazel_cmd=(
    bazel run @rules_rust//tools/rust_analyzer:discover_bazel_rust_project --
    --workspace="${workspace_dir}"
    --bazel_arg=--noshow_progress
)

# Map the graph
echo "Extracting Bazel truth for $ARCH..."
"${bazel_cmd[@]}" --bazel_arg=--platforms=//platforms:user_"$ARCH" > "${tmp_output}"
"${bazel_cmd[@]}" --bazel_arg=--platforms=//platforms:kernel_"$ARCH" >> "${tmp_output}"

# Merge the JSON
python3 - "${tmp_output}" "${tmp_json}" "${workspace_dir}" <<'PY'
import json
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
workspace_root = pathlib.Path(sys.argv[3])

merged_crates = []
module_to_global_id = {}
final_sysroot = None

try:
    content = src.read_text()
    for line in content.splitlines():
        if not line.strip().startswith('{'): continue
        obj = json.loads(line)
        if obj.get("kind") != "finished": continue

        project = obj["project"]
        if not final_sysroot:
            final_sysroot = {
                "sysroot": project.get("sysroot"),
                "sysroot_src": project.get("sysroot_src"),
                "runnables": project.get("runnables", []) # Restored to prevent VS Code extension quirks
            }

        local_to_global = {}
        run_crates = project.get("crates", [])

        # Pass 1: Resolve and Deduplicate
        for i, crate in enumerate(run_crates):
            root = crate["root_module"]
            if not pathlib.Path(root).is_absolute():
                p = workspace_root / root
            else:
                p = pathlib.Path(root)

            try:
                abs_root = str(p.resolve())
            except:
                abs_root = str(p)

            if abs_root not in module_to_global_id:
                global_id = len(merged_crates)
                module_to_global_id[abs_root] = global_id
                
                new_crate = crate.copy()
                new_crate["root_module"] = abs_root
                new_crate["deps"] = []
                merged_crates.append(new_crate)

            local_to_global[i] = module_to_global_id[abs_root]

        # Pass 2: Stitch dependencies
        for i, crate in enumerate(run_crates):
            global_idx = local_to_global[i]
            target_crate = merged_crates[global_idx]

            current_dep_ids = {d["crate"] for d in target_crate["deps"]}
            for dep in crate.get("deps", []):
                local_dep_idx = dep["crate"]
                if local_dep_idx in local_to_global:
                    global_dep_idx = local_to_global[local_dep_idx]

                    if global_dep_idx != global_idx and global_dep_idx not in current_dep_ids:
                        target_crate["deps"].append({
                            "crate": global_dep_idx,
                            "name": dep["name"]
                        })

    if final_sysroot:
        final_sysroot["crates"] = merged_crates
        with open(dst, 'w') as f:
            json.dump(final_sysroot, f, indent=2)
        sys.exit(0)
except Exception as e:
    import traceback
    traceback.print_exc()
    sys.exit(1)
PY

if [ $? -eq 0 ] && [ -s "${tmp_json}" ]; then
    mv "${tmp_json}" "${output_path}"
    echo "LSP Sync: Merged rust-project.json updated successfully." >&2
else
    echo "Error: Failed to merge projects." >&2
    exit 1
fi
