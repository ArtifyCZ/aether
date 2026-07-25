use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let link_script = match arch.as_str() {
        "aarch64" => "entry/kernel.aarch64.ld",
        "x86_64" => "entry/kernel.x86_64.ld",
        _ => panic!("unsupported target architecture {arch}"),
    };
    println!("cargo:rustc-link-arg-bin=kernel=-T{link_script}");
}
