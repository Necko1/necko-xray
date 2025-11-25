use clap::Subcommand;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::api::daemon::SOCKET_PATH;
use crate::api::Request;
use crate::config::generate_config_from_profile;
use bincode::serde::encode_to_vec;
use bincode::config::standard;

#[derive(Subcommand)]
pub enum CliCommands {
    /// Choose profile
    Profile { path: String },

    /// Show status
    #[command(subcommand)]
    Stats(StatsCommands),
}

#[derive(Subcommand)]
pub enum StatsCommands {
    /// User stats
    #[command(subcommand)]
    User(UserStatsCommands),

    /// Inbound traffic
    Inbound { tag: String },

    /// Outbound traffic
    Outbound { tag: String },

    /// System stats
    System,
}

#[derive(Subcommand)]
pub enum UserStatsCommands {
    /// Show online
    #[command(subcommand)]
    Online(UserStatsOnlineCommands),

    /// Show traffic
    Traffic { email: String },
}

#[derive(Subcommand)]
pub enum UserStatsOnlineCommands {
    /// User's ip count
    Count { email: String },

    /// User's ip list
    List { email: String },
}

pub async fn handle_command(cmd: CliCommands) -> anyhow::Result<()> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await?;

    let request: Request = match cmd {
        CliCommands::Profile { path } => {
            let _ = generate_config_from_profile(Some(&format!("profiles/{}", path)))?;
            return Ok(())
        },
        CliCommands::Stats(stats_cmd) => match stats_cmd {
            StatsCommands::User(user_cmd) => match user_cmd {
                UserStatsCommands::Online(online_cmd) => match online_cmd {
                    UserStatsOnlineCommands::Count { email } =>
                        Request::GetStatsUserOnlineCount { email },
                    UserStatsOnlineCommands::List { email } =>
                        Request::GetStatsUserOnlineIpList { email },
                },
                UserStatsCommands::Traffic { email } =>
                    Request::GetStatsUserTraffic { email },
            },
            StatsCommands::Inbound { tag } =>
                Request::GetStatsInboundTraffic { tag },
            StatsCommands::Outbound { tag } =>
                Request::GetStatsOutboundTraffic { tag },
            StatsCommands::System => Request::GetStatsSystem,
        },
    };

    let bytes = encode_to_vec(&request, standard())?;
    let len = (bytes.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    println!("{}", response);
    Ok(())
}
