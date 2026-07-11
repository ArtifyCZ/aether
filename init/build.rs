use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let c_files = match arch.as_str() {
        "aarch64" => &[
            "src/drivers/keyboard/aarch64/keyboard.c",
            "src/drivers/serial/aarch64/serial.c",
        ] as &[&str],
        "x86_64" => &[
            "src/drivers/keyboard/x86_64/keyboard.c",
            "src/drivers/keyboard/x86_64/keyboard_scancodes.c",
            "src/drivers/serial/x86_64/serial.c",
        ] as &[&str],
        _ => panic!("Unsupported target architecture: {arch}"),
    };
    let sysroot = env::var("SYSROOT").unwrap_or(format!("../build/{arch}/sysroot"));
    cc::Build::new()
        .archiver("llvm-ar")
        .pic(true)
        .include("include")
        .include("src")
        .include(format!("{sysroot}/usr/include"))
        .files(c_files)
        .compile("init_c_parts");

    // The init program uses the init-contract ABI (kernel passes `boot_info`,
    // not aether_rt's `StartupInfo`). The real entrypoint is `_entry` in
    // src/entry.rs, which bootstraps a proper stack and then calls into
    // aether_rt's `_start`. Override the linker's default `_start` entry so
    // (a) `e_entry` points at `_entry`, and (b) `_entry` is a GC root and
    // doesn't get dropped by `--gc-sections`.
    println!("cargo:rustc-link-arg-bin=init=--entry=_entry");
}
