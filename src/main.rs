use clap::{Parser, Subcommand};
use necko_xray::api::daemon;
use necko_xray::api::Request;
use necko_xray::core::CoreCommands;

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

    /// Core (API) commands
    #[command(subcommand)]
    Core(CoreCommands),
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
        Some(Commands::Core(cmd)) => {
            necko_xray::core::handle_command(cmd).await?;
        }
        None | Some(Commands::Version) => {
            println!("v{}", env!("CARGO_PKG_VERSION"));
            println!("necko-xray help")
        }
    }

    Ok(())
}