use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "kayton_kernel",
    author,
    version,
    about = "Minimal Jupyter echo kernel",
)]
pub struct Args {
    #[arg(short = 'f', long = "connection-file")]
    pub connection_file: Option<PathBuf>,

    #[arg(long)]
    pub install: bool,
}
