use crate::api::{daemon, Request};
use crate::config::generate_config_from_profile;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum XrayCommands {
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

pub async fn handle_command(cmd: XrayCommands) -> anyhow::Result<()> {
    let request: Request = match cmd {
        XrayCommands::Profile { path } => {
            let _ = generate_config_from_profile(Some(&format!("/etc/xray/profiles/{}", path)))?;
            return Ok(())
        },
        XrayCommands::Stats(stats_cmd) => match stats_cmd {
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

    let response = daemon::send_request(request).await?;
    println!("{}", response);
    
    Ok(())
}
