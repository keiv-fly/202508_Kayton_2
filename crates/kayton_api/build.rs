fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file(format!("{}/cbindgen.toml", crate_dir)).unwrap())
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}/include/kayton_api.h", crate_dir));
}
