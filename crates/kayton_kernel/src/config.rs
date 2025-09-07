use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub ip: String,
    pub transport: String,
    pub signature_scheme: String,
    pub key: String,
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
}

pub fn read_connection_file(path: &Path) -> Result<ConnectionConfig> {
    let mut file = fs::File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let cfg: ConnectionConfig = serde_json::from_str(&s)?;
    Ok(cfg)
}
