use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=syscall_kernel_gen.py");
    println!("cargo:rerun-if-changed=../api/syscalls/syscalls.toml");
    println!("cargo:rerun-if-changed=../api/syscalls/errors.toml");
    println!("cargo:rerun-if-changed=../api/syscalls/syscall_parser.py");
    println!("cargo:rerun-if-env-changed=PYTHON");

    let srctree_path = env::current_dir().unwrap().join("../..");

    let python = pick_python();
    let output = Command::new(&python)
        .arg("syscall_kernel_gen.py")
        .arg("../api/syscalls/syscalls.toml")
        .arg("../api/syscalls/errors.toml")
        .env("PYTHONPATH", &srctree_path)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {python:?}: {e}"));

    if !output.status.success() {
        panic!(
            "{python:?} syscall_kernel_gen.py exited with {}\n--- stderr ---\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::write(out_path.join("syscalls.rs"), &output.stdout).unwrap();
}

/// Locate a Python interpreter that ships `tomllib` (i.e. >= 3.11).
///
/// Why: `syscall_parser.py` uses `tomllib`, so a plain `python3` that resolves
/// to 3.9 (as macOS ships) silently produces an empty binding file.
fn pick_python() -> String {
    if let Ok(explicit) = env::var("PYTHON") {
        return explicit;
    }
    for candidate in ["python3.14", "python3.13", "python3.12", "python3.11", "python3"] {
        let ok = Command::new(candidate)
            .args(["-c", "import sys; sys.exit(0 if sys.version_info >= (3, 11) else 1)"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return candidate.to_string();
        }
    }
    panic!("no Python >= 3.11 found on PATH (needed for tomllib); set PYTHON to override");
}
