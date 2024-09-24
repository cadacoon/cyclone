fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=kernel/{}.ld", target);
    println!("cargo:rustc-link-arg=-Tkernel/{}.ld", target);

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
