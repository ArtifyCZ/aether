use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("platform_wrapper.h")
        .use_core()
        .clang_arg("-nostdinc")
        .clang_arg("-ffreestanding")
        .clang_arg("-Iinclude")
        .clang_arg("-I../../3rdparty/freestnd_c_hdrs/include")
        .clang_arg("-I../../3rdparty/limine_protocol/include")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let asm_files = match arch.as_str() {
        "aarch64" => &["src/arch/aarch64/vectors.S"] as &[&str],
        "x86_64" => &[
            "src/arch/x86_64/interrupt_entry.S",
            "src/arch/x86_64/syscall_entry.S",
        ] as &[&str],
        _ => panic!("Unsupported target architecture: {arch}"),
    };
    cc::Build::new()
        .archiver("llvm-ar")
        .files(asm_files)
        .compile("kernel_platform_asm");
}
