use anyhow::{anyhow, bail};
use crate::api::{daemon, Request};
use crate::config::generate_config_from_profile;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CoreCommands {
    /// Choose profile
    Profile { path: String },

    /// Show status
    #[command(subcommand)]
    Stats(StatsCommands),

    /// Database commands
    #[command(subcommand)]
    Database(DatabaseCommands),
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

#[derive(Subcommand)]
pub enum DatabaseCommands {
    /// Users commands
    #[command(subcommand)]
    Users(UsersCommands),
}

#[derive(Subcommand)]
pub enum UsersCommands {
    /// Create user
    Create {
        email: String,
        tags: Option<String>,
        inbounds: Option<String>,
        traffic_limit: Option<String>,
        reset_traffic_every: Option<String>,
        ip_limit: Option<i64>,
        ip_expire_after: Option<String>,
        is_active: Option<bool>,
    },

    /// Delete user
    Delete { email: String },

    /// Get all users
    Get,
}

pub async fn handle_command(cmd: CoreCommands) -> anyhow::Result<()> {
    let request: Request = match cmd {
        CoreCommands::Profile { path } => {
            let _ = generate_config_from_profile(Some(&format!("/etc/xray/profiles/{}", path)))?;
            return Ok(())
        },
        CoreCommands::Stats(stats_cmd) => match stats_cmd {
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
        CoreCommands::Database(db_cmd) => match db_cmd {
            DatabaseCommands::Users(users_cmd) => match users_cmd {
                UsersCommands::Create {
                    email,
                    tags,
                    inbounds,
                    traffic_limit,
                    reset_traffic_every,
                    ip_limit,
                    ip_expire_after,
                    is_active
                } => {
                    let traffic_limit: i64 = match traffic_limit {
                        Some(s) => {
                            let s = s.trim().to_uppercase();

                            let idx = s
                                .chars()
                                .position(|c| !c.is_ascii_digit())
                                .ok_or_else(|| anyhow!("Traffic limit must contain a value and unit"))?;

                            let (value_str, unit) = s.split_at(idx);
                            let value: u64 = value_str
                                .parse()
                                .map_err(|e| anyhow!("Invalid traffic value `{value_str}`: {e}"))?;
                            value as i64 * match unit {
                                "B" => 1,
                                "KB" => 1024,
                                "MB" => 1024 * 1024,
                                "GB" => 1024 * 1024 * 1024,
                                "TB" => 1024 * 1024 * 1024 * 1024,
                                _ => bail!("Unsupported traffic unit {}", unit),
                            }
                        }
                        None => 0,
                    };

                    let reset_traffic_every = reset_traffic_every
                        .map(|rte| crate::datetime::parse_seconds(&rte).unwrap_or(0) as i64);
                    let ip_limit = ip_limit.unwrap_or(0);
                    let ip_expire_after = ip_expire_after
                        .map(|iea| crate::datetime::parse_seconds(&iea).unwrap_or(0) as i64)
                        .unwrap_or(0);
                    let is_active = is_active.unwrap_or(true);

                    let tags: Option<Vec<String>> = tags.map(|t| t
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect());

                    let inbounds: Option<Vec<String>> = inbounds.map(|i| i
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect());

                    Request::CreateUser {
                        email,
                        tags,
                        inbounds,
                        traffic_limit,
                        reset_traffic_every,
                        expire_at: None,
                        ip_limit,
                        ip_limit_punishment: None,
                        ip_expire_after,
                        is_active,
                    }
                },
                UsersCommands::Delete { email } =>
                    Request::DeleteUser { email },
                UsersCommands::Get =>
                    Request::GetAllUsers,
            },
        },
    };

    let response = daemon::send_request(request).await?;
    println!("{}", response);
    
    Ok(())
}
