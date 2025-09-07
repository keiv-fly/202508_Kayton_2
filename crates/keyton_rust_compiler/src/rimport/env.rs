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

/// Discover the plugin DLL/SO/DYLIB path for a given module name in the active environment.
pub fn discover_plugin_dll_path(module: &str) -> Result<PathBuf> {
    let env_dir = discover_active_env_dir()?;
    let libs = env_dir.join("libs").join(module);
    // Search libs/<module>/<version>/<target> for a library file
    if libs.exists() {
        for entry in fs::read_dir(&libs)? {
            let entry = entry?;
            let ver_dir = entry.path();
            if ver_dir.is_dir() {
                for target_entry in fs::read_dir(&ver_dir)? {
                    let target_dir = target_entry?.path();
                    if target_dir.is_dir() {
                        // Prefer canonical plugin filename; fallback to first dll-like
                        #[cfg(target_os = "windows")]
                        let candidates = ["plugin.dll".to_string()];
                        #[cfg(target_os = "linux")]
                        let candidates = ["libplugin.so".to_string(), format!("lib{}.so", module)];
                        #[cfg(target_os = "macos")]
                        let candidates = [
                            "libplugin.dylib".to_string(),
                            format!("lib{}.dylib", module),
                        ];

                        for cand in &candidates {
                            let p = target_dir.join(cand);
                            if p.exists() {
                                return Ok(p);
                            }
                        }

                        // Fallback: pick the first file with dll/so/dylib extension
                        for f in fs::read_dir(&target_dir)? {
                            let f = f?.path();
                            if let Some(ext) = f.extension().and_then(|s| s.to_str()) {
                                let e = ext.to_ascii_lowercase();
                                if e == "dll" || e == "so" || e == "dylib" {
                                    return Ok(f);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    anyhow::bail!(
        "plugin library not found for module '{}' in active environment",
        module
    )
}
