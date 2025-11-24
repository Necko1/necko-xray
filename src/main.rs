mod daemon;
mod cli;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "necko-xray")]
#[command(about = "A lightweight CLI xray-core wrapper, written on rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon (default mode)
    Daemon,

    /// CLI commands
    #[command(subcommand)]
    Cli(CliCommands),
}

#[derive(Subcommand)]
enum CliCommands {
    /// Show status
    Status,
    /// Add user
    AddUser { email: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Daemon) => {
            daemon::start().await?;
        }
        Some(Commands::Cli(cmd)) => {
            cli::handle_command(cmd).await?;
        }
    }

    Ok(())
}
