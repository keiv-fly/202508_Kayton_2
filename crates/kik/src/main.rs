use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};

mod env;

#[derive(Parser, Debug)]
#[command(
    name = "kik",
    version,
    about = "Kayton environment manager",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new environment (named or local)
    Create(CreateArgs),
    /// Activate an environment in the current shell (prints PowerShell commands)
    Activate(ActivateArgs),
    /// List installed plugins and environment info
    List,
    /// Install a crate as a plugin into the active environment
    Rinstall(RinstallArgs),
    /// Uninstall a crate from the active environment
    Uninstall(UninstallArgs),
    /// Install a Jupyter kernel from this environment
    Kernel(KernelArgs),
}

#[derive(Args, Debug)]
struct CreateArgs {
    /// Environment name or 'local' for .kayton in CWD
    name: String,
}

#[derive(Args, Debug)]
struct ActivateArgs {
    /// Environment name or 'local'
    name: String,
}

#[derive(Args, Debug)]
struct RinstallArgs {
    /// Crate name
    crate_name: String,
    /// Optional crate version (semver)
    #[arg(long)]
    version: Option<String>,
    /// Cargo features
    #[arg(long)]
    features: Option<String>,
}

#[derive(Args, Debug)]
struct UninstallArgs {
    /// Crate name to uninstall
    crate_name: String,
}

#[derive(Args, Debug)]
struct KernelArgs {
    #[command(subcommand)]
    command: KernelSubcommands,
}

#[derive(Subcommand, Debug)]
enum KernelSubcommands {
    /// Install the Jupyter kernel using the active environment
    Install {
        /// Optional custom kernel name
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Create(args) => cmd_create(args),
        Commands::Activate(args) => cmd_activate(args),
        Commands::List => cmd_list(),
        Commands::Rinstall(args) => cmd_rinstall(args),
        Commands::Uninstall(args) => cmd_uninstall(args),
        Commands::Kernel(args) => cmd_kernel(args),
    }
}

fn cmd_create(args: CreateArgs) -> Result<()> {
    env::create_environment(&args.name).with_context(|| format!("create env '{}'", args.name))
}

fn cmd_activate(args: ActivateArgs) -> Result<()> {
    let activation = env::activation_for(&args.name)?;
    // Print PowerShell commands the user can eval via Invoke-Expression
    println!("$env:KAYTON_ACTIVE_ENV = '{}';", activation.env_name);
    if let Some(bin_path) = activation.prepend_path {
        // Prepend to PATH
        println!("$env:PATH = '{};' + $env:PATH;", bin_path);
    }
    Ok(())
}

fn cmd_list() -> Result<()> {
    let info = env::describe_active_env()?;
    println!("{}", info);
    Ok(())
}

fn cmd_rinstall(args: RinstallArgs) -> Result<()> {
    bail!(
        "Not yet implemented: rinstall {}{}",
        args.crate_name,
        args.version
            .as_ref()
            .map(|v| format!(" @{}", v))
            .unwrap_or_default()
    );
}

fn cmd_uninstall(args: UninstallArgs) -> Result<()> {
    bail!("Not yet implemented: uninstall {}", args.crate_name);
}

fn cmd_kernel(args: KernelArgs) -> Result<()> {
    match args.command {
        KernelSubcommands::Install { name } => {
            let display = name.unwrap_or_else(|| "kayton".to_string());
            bail!("Not yet implemented: kernel install (name='{}')", display)
        }
    }
}
