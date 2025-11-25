mod cli;
mod api;
mod config;

use clap::{Parser, Subcommand};
use api::daemon;
use crate::cli::CliCommands;

#[derive(Parser)]
#[command(name = "necko-xray")]
#[command(about = "A lightweight CLI xray-core wrapper, written on rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Start,

    /// Stop the daemon
    Stop,

    /// Current version
    Version,

    /// CLI commands
    #[command(subcommand)]
    Cli(CliCommands),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start) => {
            if let Err(e) = daemon::lock::acquire_lock() {
                eprintln!("[necko-xray]: {}", e);
                std::process::exit(1);
            }

            daemon::start().await?;
            daemon::lock::release_lock();
        }
        Some(Commands::Stop) => {
            daemon::stop().await?;
        }
        Some(Commands::Cli(cmd)) => {
            cli::handle_command(cmd).await?;
        }
        None | Some(Commands::Version) => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            println!("necko-xray help")
        }
    }

    Ok(())
}