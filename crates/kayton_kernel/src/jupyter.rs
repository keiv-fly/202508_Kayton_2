#![cfg(feature = "jupyter")]

use anyhow::Result;
use hmac::{Hmac, Mac};
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct ConnectionFile {
    pub ip: String,
    pub transport: String,
    pub key: String,
    pub signature_scheme: String,
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
}

#[derive(Serialize, Deserialize)]
struct KernelInfoReply {
    protocol_version: String,
    implementation: String,
    implementation_version: String,
    language_info: serde_json::Value,
    status: String,
}

pub fn run_kernel(connection_file: &std::path::Path) -> Result<()> {
    // Minimal placeholder: parse connection file and exit. Full ZMQ wiring not implemented here.
    let _cfg: ConnectionFile = serde_json::from_str(&fs::read_to_string(connection_file)?)?;
    // Intentionally no-op to keep build working; the full loop would use zmq sockets and HMAC.
    Ok(())
}
