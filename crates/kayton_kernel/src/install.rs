use anyhow::Result;
use serde_json::json;
use std::fs;

pub fn install_kernelspec() -> Result<()> {
    let mut base =
        dirs::data_dir().ok_or_else(|| anyhow::anyhow!("cannot determine data_dir"))?;
    base.push("jupyter");
    base.push("kernels");
    base.push("kayton");
    if base.exists() {
        fs::remove_dir_all(&base)?;
    }
    fs::create_dir_all(&base)?;

    let exe = std::env::current_exe()?;
    let argv = vec![
        exe.to_string_lossy().to_string(),
        "-f".to_string(),
        "{connection_file}".to_string(),
    ];

    let kernel_json = json!({
        "argv": argv,
        "display_name": "Kayton",
        "language": "kayton",
        "interrupt_mode": "message",
        "env": {"RUST_LOG": "info"}
    });

    let kernel_json_path = base.join("kernel.json");
    fs::write(&kernel_json_path, serde_json::to_vec_pretty(&kernel_json)?)?;

    Ok(())
}
