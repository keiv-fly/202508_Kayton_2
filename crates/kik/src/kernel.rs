use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde_json::json;

use crate::env;

/// Install the Kayton Jupyter kernel for the active environment.
/// - name: optional kernel ID/name to register with Jupyter. Defaults to "kayton".
pub fn install_kernel(name: Option<String>) -> Result<()> {
    let env_dir = env::discover_active_env().context("discover active environment")?;
    let env_name = env_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<unknown>")
        .to_string();

    // Ensure kernel binary exists inside env/bin (unless skipped)
    let bin_dir = env_dir.join("bin");
    fs::create_dir_all(&bin_dir).context("create env bin dir")?;
    let dest_kernel_exe = bin_dir.join(kernel_exe_name());
    let argv0 = if dest_kernel_exe.exists() {
        dest_kernel_exe.display().to_string()
    } else if skip_kernel_copy() {
        // Use bare executable name; assumes it will be on PATH when launched by Jupyter
        kernel_exe_name().to_string()
    } else if let Some(src) = find_kayton_kernel_exe()? {
        fs::copy(&src, &dest_kernel_exe)
            .with_context(|| format!("copy kayton_kernel to {}", dest_kernel_exe.display()))?;
        dest_kernel_exe.display().to_string()
    } else if !skip_build_kernel() {
        // Attempt to build and then try again
        let ws_root = workspace_root()
            .ok_or_else(|| anyhow!("could not determine workspace root to build kayton_kernel"))?;
        run_cmd(
            Command::new("cargo")
                .arg("build")
                .arg("-p")
                .arg("kayton_kernel")
                .arg("--release")
                .current_dir(&ws_root),
            "cargo build -p kayton_kernel --release",
        )?;
        if let Some(src) = find_kayton_kernel_exe()? {
            fs::copy(&src, &dest_kernel_exe)
                .with_context(|| format!("copy kayton_kernel to {}", dest_kernel_exe.display()))?;
            dest_kernel_exe.display().to_string()
        } else {
            return Err(anyhow!(
                "kayton_kernel executable not found after build; set KIK_SKIP_KERNEL_COPY=1 to skip copy"
            ));
        }
    } else {
        return Err(anyhow!(
            "kayton_kernel executable not found; set KIK_SKIP_KERNEL_COPY=1 to skip copy"
        ));
    };

    // Write kernelspec under env/kernels/<kernel_id>/kernel.json
    let kernel_id = name.unwrap_or_else(|| "kayton".to_string());
    let kernels_dir = env_dir.join("kernels").join(&kernel_id);
    fs::create_dir_all(&kernels_dir)
        .with_context(|| format!("create kernelspec dir {}", kernels_dir.display()))?;

    let argv = vec![argv0, "-f".to_string(), "{connection_file}".to_string()];
    let display_name = format!("Kayton ({})", &env_name);
    let mut env_map = serde_json::Map::new();
    env_map.insert("RUST_LOG".to_string(), json!("info"));
    env_map.insert("KAYTON_ACTIVE_ENV".to_string(), json!(env_name));

    let kernel_json = json!({
        "argv": argv,
        "display_name": display_name,
        "language": "kayton",
        "interrupt_mode": "message",
        "env": env_map,
    });
    let kernel_json_path = kernels_dir.join("kernel.json");
    fs::write(&kernel_json_path, serde_json::to_vec_pretty(&kernel_json)?)
        .with_context(|| format!("write {}", kernel_json_path.display()))?;

    // Register with Jupyter (best-effort). Respect env var to skip.
    if !skip_jupyter_registration() {
        let mut cmd = Command::new("jupyter");
        cmd.arg("kernelspec")
            .arg("install")
            .arg("--user")
            .arg("--replace")
            .arg("--name")
            .arg(&kernel_id)
            .arg(&kernels_dir);
        let out = cmd.output();
        match out {
            Ok(o) if o.status.success() => {
                // ok
            }
            Ok(o) => {
                let msg = String::from_utf8_lossy(&o.stderr);
                eprintln!(
                    "Warning: failed to register kernel with Jupyter (status {}): {}",
                    o.status, msg
                );
            }
            Err(_) => {
                eprintln!(
                    "Warning: 'jupyter' not found on PATH; kernelspec written but not registered."
                );
            }
        }
    }

    println!(
        "Installed Jupyter kernel '{}' at {}",
        kernel_id,
        kernels_dir.display()
    );
    Ok(())
}

/// Uninstall the kernel by name. Removes Jupyter registration and the env-local kernelspec.
pub fn uninstall_kernel(name: &str) -> Result<()> {
    // Best-effort Jupyter uninstall
    if !skip_jupyter_registration() {
        let mut cmd = Command::new("jupyter");
        cmd.arg("kernelspec").arg("uninstall").arg("-y").arg(name);
        let _ = cmd.status();
    }

    // Remove env-local spec directory if present
    let env_dir = env::discover_active_env().context("discover active environment")?;
    let spec_dir = env_dir.join("kernels").join(name);
    if spec_dir.exists() {
        fs::remove_dir_all(&spec_dir)
            .with_context(|| format!("remove kernelspec dir {}", spec_dir.display()))?;
    }
    println!("Uninstalled Jupyter kernel '{}'", name);
    Ok(())
}

fn kernel_exe_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "kayton_kernel.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "kayton_kernel"
    }
}

fn skip_build_kernel() -> bool {
    std::env::var("KIK_SKIP_BUILD_KERNEL").unwrap_or_default() == "1"
}

fn skip_kernel_copy() -> bool {
    std::env::var("KIK_SKIP_KERNEL_COPY").unwrap_or_default() == "1"
}

fn skip_jupyter_registration() -> bool {
    std::env::var("KIK_SKIP_JUPYTER").unwrap_or_default() == "1"
}

fn find_kayton_kernel_exe() -> Result<Option<PathBuf>> {
    let exe = std::env::current_exe().ok();
    if let Some(e) = exe.as_ref() {
        if let Some(dir) = e.parent() {
            let c1 = dir.join(kernel_exe_name());
            if c1.exists() {
                return Ok(Some(c1));
            }
            let c2 = dir.join("..").join(kernel_exe_name());
            if c2.exists() {
                return Ok(Some(c2));
            }
            if let Some(parent) = dir.parent() {
                let c3 = parent.join("debug").join(kernel_exe_name());
                if c3.exists() {
                    return Ok(Some(c3));
                }
                let c4 = parent.join("release").join(kernel_exe_name());
                if c4.exists() {
                    return Ok(Some(c4));
                }
            }
        }
    }

    // Try workspace target paths
    if let Some(ws) = workspace_root() {
        let c5 = ws.join("target").join("release").join(kernel_exe_name());
        if c5.exists() {
            return Ok(Some(c5));
        }
        let c6 = ws.join("target").join("debug").join(kernel_exe_name());
        if c6.exists() {
            return Ok(Some(c6));
        }
    }
    Ok(None)
}

fn run_cmd(cmd: &mut Command, human: &str) -> Result<()> {
    let status = cmd.status().with_context(|| format!("spawn {}", human))?;
    if !status.success() {
        return Err(anyhow!("command failed: {}", human));
    }
    Ok(())
}

fn workspace_root() -> Option<PathBuf> {
    // Best-effort: assume kik is inside <ws>/target/<profile>/kik(.exe). Go up until a Cargo.toml is found.
    let exe = std::env::current_exe().ok()?;
    let mut p = exe.clone();
    for _ in 0..6 {
        if let Some(parent) = p.parent() {
            p = parent.to_path_buf();
            if p.join("Cargo.toml").exists() {
                return Some(p);
            }
        }
    }
    // Fallback to CWD
    let cwd = std::env::current_dir().ok()?;
    if cwd.join("Cargo.toml").exists() {
        Some(cwd)
    } else {
        None
    }
}
