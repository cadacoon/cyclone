fn main() {
    bindgen::Builder::default()
        .use_core()
        .header("source/include/acpi.h")
        .clang_arg("-D_LINUX")
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("acpi.rs"))
        .expect("Failed to write bindings");
}
