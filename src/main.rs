mod cli;
mod api;
mod config;

use clap::{Parser, Subcommand};
use api::daemon;
use crate::api::Request;
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

    /// Restart the daemon
    Restart,

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
            if daemon::lock::is_daemon_running() {
                let resp = daemon::send_request(Request::StartXray).await?;
                println!("[necko-xray]: {}", resp);
                return Ok(());
            }

            if let Err(e) = daemon::lock::acquire_lock() {
                eprintln!("[necko-xray]: {}", e);
                std::process::exit(1);
            }

            daemon::start().await?;
        }
        Some(Commands::Stop) => {
            let resp = daemon::send_request(Request::StopXray).await?;
            println!("[necko-xray]: {}", resp);
        }
        Some(Commands::Restart) => {
            if daemon::lock::is_daemon_running() {
                let resp = daemon::send_request(Request::RestartXray).await?;
                println!("[necko-xray]: {}", resp);
            } else {
                eprintln!("[necko-xray]: Daemon is not running");
                std::process::exit(1);
            }
        }
        Some(Commands::Cli(cmd)) => {
            cli::handle_command(cmd).await?;
        }
        None | Some(Commands::Version) => {
            println!("v{}", env!("CARGO_PKG_VERSION"));
            println!("necko-xray help")
        }
    }

    Ok(())
}