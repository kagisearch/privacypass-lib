use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("KAGIPP_FFI_H")
        .with_documentation(true)
        .with_style(cbindgen::Style::Both)
        .with_no_includes()
        .with_sys_include("stdint.h")
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file("include/kagipp_ffi.h");
}
