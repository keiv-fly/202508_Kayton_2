use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::env;

pub fn run_install_spec(spec: &str) -> Result<()> {
    let args = parse_install_spec(spec)?;
    run(args)
}

pub fn parse_install_spec(spec: &str) -> Result<InstallArgs> {
    let s = spec.trim();
    if s.is_empty() {
        return Err(anyhow!("empty install spec"));
    }
    let bytes = s.as_bytes();
    let mut i = 0usize;
    // Parse crate name up to '[' or '='
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '[' || c == '=' || c.is_whitespace() {
            break;
        }
        i += 1;
    }
    let crate_name = s[..i].to_string();
    if crate_name.is_empty() {
        return Err(anyhow!("missing crate name in install spec"));
    }
    // Skip whitespace
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    // Optional features in [a,b,c]
    let mut features: Option<String> = None;
    if i < bytes.len() && (bytes[i] as char) == '[' {
        i += 1; // skip '['
        let start = i;
        // find matching ']'
        let mut found = None;
        while i < bytes.len() {
            if (bytes[i] as char) == ']' {
                found = Some(i);
                break;
            }
            i += 1;
        }
        let end = found.ok_or_else(|| anyhow!("missing closing ']' for features list"))?;
        let raw = &s[start..end];
        let list: Vec<String> = raw
            .split(',')
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect();
        if !list.is_empty() {
            features = Some(list.join(","));
        }
        i = end + 1; // skip ']'
                     // Skip whitespace
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
    }
    // Optional version starting with '=='
    let mut version: Option<String> = None;
    if i + 1 < bytes.len() && (bytes[i] as char) == '=' && (bytes[i + 1] as char) == '=' {
        i += 2; // skip '=='
                // Skip whitespace
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return Err(anyhow!("missing version after '=='"));
        }
        let ver = s[i..].trim();
        if ver.is_empty() {
            return Err(anyhow!("missing version after '=='"));
        }
        version = Some(ver.to_string());
        i = s.len();
    }
    // Remaining must be only whitespace
    if i < bytes.len() && !s[i..].trim().is_empty() {
        return Err(anyhow!("unexpected trailing characters in install spec"));
    }
    Ok(InstallArgs {
        crate_name,
        version,
        features,
    })
}

#[derive(Debug, Clone)]
pub struct InstallArgs {
    pub crate_name: String,
    pub version: Option<String>,
    pub features: Option<String>,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let env_dir = env::discover_active_env().context("discover active environment")?;
    let meta = env::load_env_metadata(&env_dir).context("read env metadata")?;

    // Resolve version (best-effort). If not provided, try `cargo search` and parse the first match.
    let resolved_version = match &args.version {
        Some(v) => v.clone(),
        None => resolve_crate_version(&args.crate_name).unwrap_or_else(|| "*".to_string()),
    };

    // Create a temporary working directory inside the environment.
    let tmp_root = env_dir.join("tmp");
    fs::create_dir_all(&tmp_root).context("create tmp dir in env")?;
    let work_dir = tmp_root.join(format!(
        "kik_rinstall_{}_{}",
        &args.crate_name,
        std::process::id()
    ));
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir).ok();
    }
    fs::create_dir_all(&work_dir).context("create work dir")?;

    // Scaffold wrapper crate
    let wrapper_name = format!("kayton_plugin_{}", &args.crate_name);
    let wrapper_dir = work_dir.join(&wrapper_name);
    fs::create_dir_all(wrapper_dir.join("src")).context("create wrapper src dir")?;

    write_wrapper_cargo_toml(
        &wrapper_dir,
        &wrapper_name,
        &args.crate_name,
        &resolved_version,
        args.features.as_deref(),
    )?;
    write_wrapper_lib_rs(&wrapper_dir, &args.crate_name, &resolved_version)?;

    // Configure and attempt to vendor dependencies for reproducibility
    write_vendor_config(&wrapper_dir).ok();
    let _ = run_cmd(
        Command::new("cargo")
            .arg("vendor")
            .current_dir(&wrapper_dir),
        "cargo vendor",
    );

    // Build the plugin using nightly (Rust ABI required). This will fail if nightly isn't installed.
    let _ = ensure_nightly_toolchain();
    run_cmd(
        Command::new("cargo")
            .arg("+nightly")
            .arg("build")
            .arg("--release")
            .current_dir(&wrapper_dir),
        "cargo +nightly build --release",
    )?;

    // Determine built DLL path
    let built_dll = find_built_dll(&wrapper_dir).context("locate built plugin dll")?;

    // Read resolved crate version from Cargo.lock if available (for registry path/version recording)
    let final_version =
        read_version_from_lock(&wrapper_dir, &args.crate_name).unwrap_or(resolved_version.clone());

    // Prepare final install directory
    let final_dir = env_dir
        .join("libs")
        .join(&args.crate_name)
        .join(&final_version)
        .join(&meta.target_triple);
    fs::create_dir_all(&final_dir).context("create final install dir")?;

    // Copy DLL and manifest.json
    let dll_dest = final_dir.join("plugin.dll");
    fs::copy(&built_dll, &dll_dest)
        .with_context(|| format!("copy dll to {}", dll_dest.display()))?;

    let manifest_json = wrapper_dir.join("manifest.json");
    if manifest_json.exists() {
        fs::copy(&manifest_json, final_dir.join("manifest.json")).ok();
    } else {
        // Write a minimal manifest.json for bookkeeping
        let json_body = format!(
            "{{\n  \"abi_version\": {},\n  \"crate_name\": \"{}\",\n  \"crate_version\": \"{}\",\n  \"functions\": [],\n  \"types\": []\n}}\n",
            kayton_plugin_sdk::KAYTON_PLUGIN_ABI_VERSION,
            &args.crate_name,
            &final_version
        );
        fs::write(final_dir.join("manifest.json"), json_body).context("write manifest.json")?;
    }

    // Update registry
    let mut registry = env::load_registry(&env_dir).context("load registry.json")?;
    // remove any existing entries for this crate+target
    registry
        .plugins
        .retain(|p| !(p.name == args.crate_name && p.target_triple == meta.target_triple));
    registry.plugins.push(env::PluginEntry {
        name: args.crate_name.clone(),
        version: final_version.clone(),
        target_triple: meta.target_triple.clone(),
    });
    env::save_registry(&env_dir, &registry).context("save registry.json")?;

    // Best-effort cleanup
    let _ = fs::remove_dir_all(&work_dir);

    Ok(())
}

fn resolve_crate_version(crate_name: &str) -> Option<String> {
    // cargo search <crate> output example: `serde = "1.0.210"` Description
    let output = Command::new("cargo")
        .arg("search")
        .arg(crate_name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with(&format!("{} = \"", crate_name)) {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn write_wrapper_cargo_toml(
    dir: &Path,
    wrapper_name: &str,
    crate_name: &str,
    crate_version: &str,
    features: Option<&str>,
) -> Result<()> {
    let ws_root = workspace_root()?;
    let sdk_path = ws_root.join("crates").join("kayton_plugin_sdk");
    let api_path = ws_root.join("crates").join("kayton_api");
    let mut toml = String::new();
    toml.push_str("[package]\n");
    toml.push_str(&format!("name = \"{}\"\n", wrapper_name));
    toml.push_str("version = \"0.0.0\"\n");
    toml.push_str("edition = \"2021\"\n\n");
    toml.push_str("[lib]\ncrate-type = [\"dylib\"]\n\n");
    toml.push_str("[dependencies]\n");
    toml.push_str(&format!(
        "kayton_plugin_sdk = {{ path = \"{}\" }}\n",
        pathdiff::diff_paths(&sdk_path, dir)
            .unwrap_or(sdk_path.clone())
            .display()
    ));
    toml.push_str(&format!(
        "kayton_api = {{ path = \"{}\" }}\n",
        pathdiff::diff_paths(&api_path, dir)
            .unwrap_or(api_path.clone())
            .display()
    ));
    if let Some(f) = features {
        toml.push_str(&format!(
            "{} = {{ version = \"{}\", features = [{}] }}\n",
            crate_name,
            crate_version,
            f.split(',')
                .map(|s| format!("\"{}\"", s.trim()))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        toml.push_str(&format!("{} = \"{}\"\n", crate_name, crate_version));
    }
    fs::write(dir.join("Cargo.toml"), toml).context("write wrapper Cargo.toml")?;
    Ok(())
}

fn write_wrapper_lib_rs(dir: &Path, crate_name: &str, crate_version: &str) -> Result<()> {
    let lib_rs = dir.join("src").join("lib.rs");
    let mut f = fs::File::create(&lib_rs).context("create wrapper lib.rs")?;
    writeln!(
        f,
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        "#![feature(abi_rust)]",
        "use kayton_plugin_sdk::{Manifest, kayton_manifest, kayton_exports};",
        "use kayton_api::KaytonContext;",
        &format!(
            "static MANIFEST: Manifest = kayton_manifest!(\n    crate_name = \"{}\",\n    crate_version = \"{}\",\n    functions = [],\n    types = []\n);",
            crate_name, crate_version
        ),
        "fn register(_ctx: &mut KaytonContext) {}",
        "kayton_exports!(MANIFEST, register);",
        "// Also write manifest.json alongside for kik to copy",
        &format!(
            "#[allow(dead_code)] fn _emit_manifest_file() {{ let bytes = kayton_plugin_sdk::manifest::Manifest {{ abi_version: kayton_plugin_sdk::KAYTON_PLUGIN_ABI_VERSION, crate_name: \"{}\".into(), crate_version: \"{}\".into(), functions: alloc::vec::Vec::new(), types: alloc::vec::Vec::new() }}.to_json_bytes(); let _ = std::fs::write(\"manifest.json\", &bytes); }}",
            crate_name, crate_version
        ),
        "#[used] static _FORCE_EMIT: extern \"C\" fn() = { _emit_manifest_file as extern \"C\" fn() };"
    )
    .ok();
    Ok(())
}

fn run_cmd(cmd: &mut Command, human: &str) -> Result<()> {
    let status = cmd.status().with_context(|| format!("spawn {}", human))?;
    if !status.success() {
        return Err(anyhow!("command failed: {}", human));
    }
    Ok(())
}

fn write_vendor_config(wrapper_dir: &Path) -> Result<()> {
    let cargo_dir = wrapper_dir.join(".cargo");
    fs::create_dir_all(&cargo_dir).context("create .cargo dir")?;
    let cfg = cargo_dir.join("config.toml");
    let body = "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n";
    fs::write(cfg, body).context("write .cargo/config.toml")?;
    Ok(())
}

fn ensure_nightly_toolchain() -> Result<()> {
    // If rustup is present and nightly missing, try to install.
    let list = Command::new("rustup").arg("toolchain").arg("list").output();
    if let Ok(out) = list {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.contains("nightly") {
                let _ = Command::new("rustup")
                    .arg("toolchain")
                    .arg("install")
                    .arg("nightly")
                    .arg("-y")
                    .status();
            }
        }
    }
    Ok(())
}

fn find_built_dll(wrapper_dir: &Path) -> Option<PathBuf> {
    let p = wrapper_dir.join("target").join("release");
    if !p.exists() {
        return None;
    }
    // Windows DLL name equals crate name, but we don't know it here. Find any .dll newer than others.
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(&p).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "dll" {
                let meta = entry.metadata().ok()?;
                let mtime = meta.modified().ok()?;
                match &newest {
                    Some((t, _)) if *t >= mtime => {}
                    _ => newest = Some((mtime, path.clone())),
                }
            }
        }
    }
    newest.map(|(_, p)| p)
}

fn read_version_from_lock(wrapper_dir: &Path, crate_name: &str) -> Option<String> {
    let lock_path = wrapper_dir.join("Cargo.lock");
    let s = fs::read_to_string(lock_path).ok()?;
    let mut current_name: Option<String> = None;
    for line in s.lines() {
        if line.starts_with("name = \"") {
            let name = line
                .trim_start_matches("name = \"")
                .trim_end_matches('\"')
                .to_string();
            current_name = Some(name);
        } else if line.starts_with("version = \"") {
            if current_name.as_deref() == Some(crate_name) {
                let v = line
                    .trim_start_matches("version = \"")
                    .trim_end_matches('\"')
                    .to_string();
                return Some(v);
            }
        }
    }
    None
}

fn workspace_root() -> Result<PathBuf> {
    // Assume kik is inside <ws>/crates/kik, so go up two directories
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let mut p = exe.clone();
    // Strip target/debug/kik.exe -> ... -> workspace root best-effort
    for _ in 0..4 {
        if let Some(parent) = p.parent() {
            p = parent.to_path_buf();
        }
    }
    // Fallback to CWD if not a workspace
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if p.join("Cargo.toml").exists() {
        Ok(p)
    } else {
        Ok(cwd)
    }
}
