use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use kayton_interactive_shared::{InteractiveState, execute_prepared, prepare_input};
use log::{info, warn};

mod jupyter;

#[derive(Parser, Debug)]
#[command(version, about = "Kayton Jupyter kernel (experimental)")]
struct Args {
    #[arg(short = 'f', long = "connection-file", value_name = "FILE")]
    connection_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Initialize logging with more detail
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();

    info!("kayton_kernel starting");

    let args = Args::parse();

    // Check if running in Jupyter mode
    if let Some(cf) = args.connection_file.as_ref() {
        info!(
            "Starting Jupyter protocol with connection file: {}",
            cf.display()
        );

        // Verify connection file exists and is readable
        if !cf.exists() {
            warn!("Connection file does not exist: {}", cf.display());
            return Err(anyhow::anyhow!("Connection file not found"));
        }

        return jupyter::run_kernel(cf);
    }

    // Fallback to stdin mode for testing
    info!("Running in stdin mode");
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

    let globals = state.vm_mut().read_all_globals_as_strings();
    let json = serde_json::json!({
        "status": "ok",
        "globals": globals,
    });
    println!("{}", serde_json::to_string(&json).unwrap());
    Ok(())
}
