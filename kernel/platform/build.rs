use std::env;
use std::fs::create_dir_all;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .use_core()
        .header("platform_wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .clang_arg(format!("--target={}", target))
        .clang_arg("-nostdinc")
        .clang_arg("-std=gnu11")
        .clang_arg("-ffreestanding")
        .clang_arg("-fno-PIC")
        .clang_arg("-ffunction-sections")
        .clang_arg("-fdata-sections")
        .clang_arg("-I./include")
        .clang_arg("-I./../../bazel-aether/external/+git_repository+freestnd-c-hdrs/include")
        .clang_arg("-I./../../bazel-aether/external/+git_repository+limine_protocol/include")
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_path)
        .expect("Couldn't write bindings!");
}
