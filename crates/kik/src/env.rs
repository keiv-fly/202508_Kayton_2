use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMetadata {
    pub kayton_version: String,
    pub abi_version: u32,
    pub target_triple: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryIndex {
    pub plugins: Vec<PluginEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    pub name: String,
    pub version: String,
    pub target_triple: String,
}

pub struct ActivationScript {
    pub env_name: String,
    pub prepend_path: Option<String>,
}

pub fn kayton_root_dir() -> Result<PathBuf> {
    let local_app = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not determine %LOCALAPPDATA% directory"))?;
    Ok(local_app.join("Kayton"))
}

pub fn envs_root_dir() -> Result<PathBuf> {
    Ok(kayton_root_dir()?.join("envs"))
}

pub fn named_env_dir(name: &str) -> Result<PathBuf> {
    Ok(envs_root_dir()?.join(name))
}

pub fn local_env_dir(cwd: &Path) -> PathBuf {
    cwd.join(".kayton")
}

pub fn create_environment(name: &str) -> Result<()> {
    if name == "local" {
        let cwd = env::current_dir().context("get current dir")?;
        create_env_layout(&local_env_dir(&cwd))
    } else {
        create_env_layout(&named_env_dir(name)?)
    }
}

fn create_env_layout(env_dir: &Path) -> Result<()> {
    let bin = env_dir.join("bin");
    let toolchain = env_dir.join("toolchain");
    let libs = env_dir.join("libs");
    let metadata = env_dir.join("metadata");
    let kernels = env_dir.join("kernels");
    fs::create_dir_all(&bin).context("create bin dir")?;
    fs::create_dir_all(&toolchain).context("create toolchain dir")?;
    fs::create_dir_all(&libs).context("create libs dir")?;
    fs::create_dir_all(&metadata).context("create metadata dir")?;
    fs::create_dir_all(&kernels).context("create kernels dir")?;

    // Minimal metadata placeholders
    let env_json = metadata.join("env.json");
    if !env_json.exists() {
        let meta = EnvMetadata {
            kayton_version: env!("CARGO_PKG_VERSION").to_string(),
            abi_version: 1,
            target_triple: std::env::var("CARGO_BUILD_TARGET")
                .unwrap_or_else(|_| "x86_64-pc-windows-msvc".to_string()),
        };
        let contents = serde_json::to_string_pretty(&meta)?;
        fs::write(env_json, contents).context("write env.json")?;
    }
    let registry_json = metadata.join("registry.json");
    if !registry_json.exists() {
        let idx = RegistryIndex { plugins: vec![] };
        fs::write(registry_json, serde_json::to_string_pretty(&idx)?)
            .context("write registry.json")?;
    }
    Ok(())
}

pub fn activation_for(name: &str) -> Result<ActivationScript> {
    if name == "local" {
        let cwd = env::current_dir().context("get current dir")?;
        let dir = local_env_dir(&cwd);
        if !dir.exists() {
            return Err(anyhow!(
                "Local environment not found at '{}' — run 'kik create local' first",
                dir.display()
            ));
        }
        Ok(ActivationScript {
            env_name: "local".to_string(),
            prepend_path: Some(dir.join("bin").display().to_string()),
        })
    } else {
        let dir = named_env_dir(name)?;
        if !dir.exists() {
            return Err(anyhow!(
                "Environment '{}' not found at '{}' — run 'kik create {}' first",
                name,
                dir.display(),
                name
            ));
        }
        Ok(ActivationScript {
            env_name: name.to_string(),
            prepend_path: Some(dir.join("bin").display().to_string()),
        })
    }
}

pub fn discover_active_env() -> Result<PathBuf> {
    if let Ok(name) = std::env::var("KAYTON_ACTIVE_ENV") {
        if name == "local" {
            let cwd = env::current_dir().context("get current dir")?;
            let dir = local_env_dir(&cwd);
            if dir.exists() {
                return Ok(dir);
            }
        } else {
            let dir = named_env_dir(&name)?;
            if dir.exists() {
                return Ok(dir);
            }
        }
    }
    // Fallbacks: project-local if exists, else base
    let cwd = env::current_dir().context("get current dir")?;
    let local = local_env_dir(&cwd);
    if local.exists() {
        return Ok(local);
    }
    // base env
    named_env_dir("base")
}

pub fn describe_active_env() -> Result<String> {
    let dir = discover_active_env()?;
    let meta_path = dir.join("metadata").join("env.json");
    let reg_path = dir.join("metadata").join("registry.json");

    let mut out = String::new();
    out.push_str(&format!(
        "Environment: {}\n",
        dir.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>")
    ));
    out.push_str(&format!("Path: {}\n", dir.display()));

    match fs::read_to_string(&meta_path) {
        Ok(s) => match serde_json::from_str::<EnvMetadata>(&s) {
            Ok(m) => {
                out.push_str(&format!(
                    "Kayton: {}  ABI: {}  Target: {}\n",
                    m.kayton_version, m.abi_version, m.target_triple
                ));
            }
            Err(_) => {
                out.push_str("env.json: <invalid>\n");
            }
        },
        Err(_) => out.push_str("env.json: <missing>\n"),
    }

    match fs::read_to_string(&reg_path) {
        Ok(s) => match serde_json::from_str::<RegistryIndex>(&s) {
            Ok(idx) => {
                if idx.plugins.is_empty() {
                    out.push_str("Plugins: <none>\n");
                } else {
                    out.push_str("Plugins:\n");
                    for p in idx.plugins {
                        out.push_str(&format!(
                            "  - {} {} [{}]\n",
                            p.name, p.version, p.target_triple
                        ));
                    }
                }
            }
            Err(_) => out.push_str("registry.json: <invalid>\n"),
        },
        Err(_) => out.push_str("registry.json: <missing>\n"),
    }

    Ok(out)
}
