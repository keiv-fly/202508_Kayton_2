use anyhow::Result;
use kayton_plugin_sdk::manifest::Manifest;
use std::fs;
use std::path::PathBuf;

// Discover the active environment directory using kik's rules
pub fn discover_active_env_dir() -> Result<PathBuf> {
    // Order: KAYTON_ACTIVE_ENV named/local, then project-local .kayton, else base
    if let Ok(name) = std::env::var("KAYTON_ACTIVE_ENV") {
        if name == "local" {
            let cwd = std::env::current_dir()?;
            let dir = cwd.join(".kayton");
            if dir.exists() {
                return Ok(dir);
            }
        } else {
            if let Some(local_app) = dirs::data_local_dir() {
                let dir = local_app.join("Kayton").join("envs").join(&name);
                if dir.exists() {
                    return Ok(dir);
                }
            }
        }
    }
    let cwd = std::env::current_dir()?;
    let local = cwd.join(".kayton");
    if local.exists() {
        return Ok(local);
    }
    // base environment
    if let Some(local_app) = dirs::data_local_dir() {
        return Ok(local_app.join("Kayton").join("envs").join("base"));
    }
    anyhow::bail!("Could not determine active Kayton environment directory")
}

pub fn load_active_env_registry() -> Result<PathBuf> {
    let dir = discover_active_env_dir()?;
    let metadata = dir.join("metadata");
    let registry = metadata.join("registry.json");
    // Optional: validate exists
    let _ = fs::metadata(&registry)?;
    Ok(dir)
}

pub fn load_plugin_manifest(module: &str) -> Result<Manifest> {
    let env_dir = discover_active_env_dir()?;
    let libs = env_dir.join("libs").join(module);
    // Pick latest/versioned folder if multiple; for now, choose any manifest.json
    let mut manifest_path: Option<PathBuf> = None;
    if libs.exists() {
        for entry in fs::read_dir(&libs)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // target triple level
                for version_entry in fs::read_dir(&path)? {
                    let version_entry = version_entry?;
                    let vp = version_entry.path();
                    if vp.is_dir() {
                        let m = vp.join("manifest.json");
                        if m.exists() {
                            manifest_path = Some(m);
                            break;
                        }
                    }
                }
                if manifest_path.is_some() {
                    break;
                }
            }
        }
    }
    let mpath = manifest_path.ok_or_else(|| {
        anyhow::anyhow!(
            "manifest.json not found for module '{}' in active environment",
            module
        )
    })?;
    let bytes = fs::read(&mpath)?;
    let mani: Manifest = serde_json::from_slice(&bytes)?;
    Ok(mani)
}
