fn main() {
    println!("cargo:rerun-if-changed=i386-unknown-none.ld");
    println!("cargo:rustc-link-arg=-Ti386-unknown-none.ld");

    bindgen::Builder::default()
        .use_core()
        .header("include/multiboot.h")
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(
            std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("multiboot.rs"),
        )
        .expect("Failed to write bindings");
}
