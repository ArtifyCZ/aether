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

bazel_cmd=(
    bazel run @rules_rust//tools/rust_analyzer:discover_bazel_rust_project --
    --workspace="${workspace_dir}"
    --bazel_arg=--noshow_progress
)

# Run for both platforms and capture both outputs
"${bazel_cmd[@]}" --bazel_arg=--platforms=//platforms:user_"$ARCH" > "${tmp_output}"
"${bazel_cmd[@]}" --bazel_arg=--platforms=//platforms:kernel_"$ARCH" >> "${tmp_output}"

python3 - "${tmp_output}" "${tmp_json}" <<'PY'
import json
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])

merged_crates = []
# Map root_module -> index in merged_crates
module_to_id = {}
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
                "runnables": project.get("runnables", [])
            }

        # Local map for this run's indices to our new global indices
        local_to_global = {}
        run_crates = project.get("crates", [])

        # First pass: Register all crates and map indices
        for i, crate in enumerate(run_crates):
            root = crate["root_module"]
            if root not in module_to_id:
                module_to_id[root] = len(merged_crates)
                merged_crates.append(crate)
            local_to_global[i] = module_to_id[root]

        # Second pass: Update dependencies for crates we just added/updated
        for i, crate in enumerate(run_crates):
            global_id = local_to_global[i]
            new_deps = []
            for dep in crate.get("deps", []):
                if dep["crate"] in local_to_global:
                    new_deps.append({
                        "crate": local_to_global[dep["crate"]],
                        "name": dep["name"]
                    })
            merged_crates[global_id]["deps"] = new_deps

    if final_sysroot:
        final_sysroot["crates"] = merged_crates
        with open(dst, 'w') as f:
            json.dump(final_sysroot, f, indent=2)
        sys.exit(0)
except Exception as e:
    print(f"Merge error: {e}", file=sys.stderr)
sys.exit(1)
PY

if [ $? -eq 0 ] && [ -s "${tmp_json}" ]; then
    mv "${tmp_json}" "${output_path}"
    echo "LSP Sync: Merged rust-project.json updated successfully." >&2
else
    echo "Error: Failed to merge projects." >&2
    exit 1
fi
