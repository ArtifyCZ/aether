#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$BUILD_WORKSPACE_DIRECTORY"
output_path="${workspace_dir}/rust-project.json"

cd "${workspace_dir}"

tmp_output="$(mktemp "${TMPDIR:-/tmp}/rust-project.raw.XXXXXX")"
tmp_json="$(mktemp "${TMPDIR:-/tmp}/rust-project.json.XXXXXX")"

cleanup() {
    rm -f "${tmp_output}" "${tmp_json}"
}
trap cleanup EXIT

# Start the command array
bazel_cmd=(
    bazel run @rules_rust//tools/rust_analyzer:discover_bazel_rust_project --
    --workspace="${workspace_dir}"
)

# Logic: If you pass arguments (like --bazel_arg=--platforms=//...), use those.
# Otherwise, default to x86_64.
if [[ $# -gt 0 ]]; then
    bazel_cmd+=("$@")
else
    bazel_cmd+=("--bazel_arg=--platforms=//platforms:kernel_x86_64")
fi

# Run it. Redirect stderr to the console (so you see progress) 
# and stdout to the file (for parsing).
"${bazel_cmd[@]}" --bazel_arg=--noshow_progress > "${tmp_output}"

python3 - "${tmp_output}" "${tmp_json}" <<'PY'
import json
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])

project = None
try:
    content = src.read_text()
    # Bazel 7+ Bzlmod output can sometimes be one giant blob or separate lines.
    # We try parsing lines first, then the whole file.
    lines = content.splitlines()
    for line in lines:
        line = line.strip()
        if not line or not line.startswith('{'): continue
        try:
            obj = json.loads(line)
            if obj.get("kind") == "finished" and "project" in obj:
                project = obj["project"]
        except: continue
except Exception as e:
    print(f"Python error: {e}", file=sys.stderr)

if project:
    with open(dst, 'w') as f:
        json.dump(project, f, indent=2)
    sys.exit(0)
else:
    print("Could not find project in output.", file=sys.stderr)
    sys.exit(1)
PY

# Simplified Bash check: if Python exited 0, the file is good.
if [ $? -eq 0 ] && [ -s "${tmp_json}" ]; then
    mv "${tmp_json}" "${output_path}"
    echo "LSP Sync: rust-project.json updated successfully." >&2
else
    echo "Error: Python failed to extract project or file is empty." >&2
    exit 1
fi
