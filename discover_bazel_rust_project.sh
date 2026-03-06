#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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
"${bazel_cmd[@]}" > "${tmp_output}" 2> >(tee /dev/stderr >&2)

python3 - "${tmp_output}" "${tmp_json}" <<'PY'
import json
import pathlib
import sys

src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])

project = None
try:
    content = src.read_text()
    for raw_line in content.splitlines():
        line = raw_line.strip()
        if not line.startswith("{"):
            continue
        try:
            obj = json.loads(line)
            if obj.get("kind") == "finished" and "project" in obj:
                project = obj["project"]
                break
        except json.JSONDecodeError:
            continue
except Exception as e:
    print(f"Extraction error: {e}", file=sys.stderr)

if project is None:
    sys.exit(1)

dst.write_text(json.dumps(project, indent=2) + "\n")
PY

if [ -f "${tmp_json}" ] && jq . "${tmp_json}" >/dev/null 2>&1; then
    mv "${tmp_json}" "${output_path}"
    # Use stderr for the success message to keep the terminal clean
    echo "LSP Sync: rust-project.json updated successfully." >&2
else
    echo "Error: Failed to generate a valid rust-project.json" >&2
    exit 1
fi
