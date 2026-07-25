use std::env;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let link_script = match arch.as_str() {
        "aarch64" => "entry/atom.aarch64.ld",
        "x86_64" => "entry/atom.x86_64.ld",
        _ => panic!("unsupported target architecture {arch}"),
    };
    println!("cargo:rustc-link-arg-bin=atom=-T{link_script}");
    println!("cargo:rerun-if-changed={link_script}");
}
