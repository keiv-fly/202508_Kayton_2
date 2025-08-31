use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Where to write the kernelspec
    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Prefer explicit CARGO_TARGET_DIR if set; otherwise assume workspace target at ../../target
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            manifest_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("target")
        });

    let spec_dir = target_dir
        .join(&profile)
        .join("kayton_kernelspec")
        .join("kayton");

    let _ = fs::create_dir_all(&spec_dir);

    // Minimal kernelspec. We rely on kayton_kernel being on PATH.
    // Jupyter will pass the connection file at {connection_file}
    let kernel_json = serde_json::json!({
        "argv": ["kayton_kernel", "-f", "{connection_file}"],
        "display_name": "Kayton",
        "language": "kayton"
    });

    let _ = fs::write(
        spec_dir.join("kernel.json"),
        serde_json::to_string_pretty(&kernel_json).unwrap_or_else(|_| String::from("{}")),
    );

    // Avoid reruns unless this file changes
    println!("cargo:rerun-if-changed=build.rs");
}
