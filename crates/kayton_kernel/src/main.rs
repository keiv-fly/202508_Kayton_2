mod args;
mod config;
mod signing;
mod protocol;
mod iopub;
mod install;
mod kernel;

use anyhow::{Context, Result};
use clap::Parser;

use args::Args;
use config::read_connection_file;
use install::install_kernelspec;
use kernel::run_kernel;

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if args.install {
        install_kernelspec().context("failed to install kernelspec")?;
        println!("Installed kayton_kernel kernelspec.");
        return Ok(());
    }

    let Some(connection_file) = args.connection_file else {
        eprintln!(
            "No connection file specified. Run with --install to install kernelspec, or pass -f <connection_file> to run."
        );
        std::process::exit(2);
    };

    let cfg = read_connection_file(&connection_file).context("failed to read connection file")?;
    run_kernel(&cfg)
}
