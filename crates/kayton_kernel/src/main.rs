use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};
#[cfg(feature = "jupyter")]
use log::info;

#[cfg(feature = "jupyter")]
mod jupyter;

#[derive(Parser, Debug)]
#[command(version, about = "Kayton Jupyter kernel (experimental)")]
struct Args {
    #[arg(short = 'f', long = "connection-file", value_name = "FILE")]
    connection_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Initialize logging (no-op if already set); respects RUST_LOG
    #[cfg(feature = "jupyter")]
    {
        let _ = env_logger::builder().format_timestamp_millis().try_init();
        info!("kayton_kernel starting (jupyter feature enabled)");
    }
    // If a Jupyter connection file is provided and the feature is enabled, run protocol loop
    #[cfg(feature = "jupyter")]
    let args = Args::parse();
    #[cfg(feature = "jupyter")]
    if let Some(cf) = args.connection_file.as_ref() {
        return jupyter::run_kernel(cf);
    }

    // For now: accept entire cell source on stdin, execute via shared engine, then
    // print a JSON object with globals snapshot for display.
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut state = InteractiveState::new();
    if !input.trim().is_empty() {
        let prep = match prepare_input(&mut state, input.trim_end_matches(['\n', '\r'])) {
            Ok(p) => p,
            Err(e) => {
                println!(
                    "{{\"status\":\"error\",\"error\":{}}}",
                    serde_json::to_string(&e.to_string())
                        .unwrap_or_else(|_| "\"unknown\"".to_string())
                );
                return Ok(());
            }
        };
        if let Err(e) = execute_prepared(&mut state, &prep) {
            println!(
                "{{\"status\":\"error\",\"error\":{}}}",
                serde_json::to_string(&e.to_string()).unwrap_or_else(|_| "\"unknown\"".to_string())
            );
            return Ok(());
        }
    }

    // Report all globals as strings for now
    let globals = state.vm_mut().read_all_globals_as_strings();
    let json = serde_json::json!({
        "status": "ok",
        "globals": globals,
    });
    println!("{}", serde_json::to_string(&json).unwrap());
    Ok(())
}
