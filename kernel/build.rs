fn main() {
    println!("cargo:rerun-if-changed=kernel/x86.ld");
    println!("cargo:rustc-link-arg=-Tkernel/x86.ld");

    bindgen::Builder::default()
        .use_core()
        .header("multiboot.h")
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(
            std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("multiboot.rs"),
        )
        .expect("Failed to write bindings");
}
