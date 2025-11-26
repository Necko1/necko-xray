use anyhow::bail;
use crate::data::postgres::types::{CreateUser, IpLimitPunishment};
use crate::proto::app::stats::command::SysStatsResponseSerializable;
use crate::Client;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod daemon;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    StartXray,
    StopXray,
    RestartXray,

    GetStatsUserOnlineCount { email: String },
    GetStatsUserOnlineIpList { email: String },
    GetStatsUserTraffic { email: String },
    GetStatsInboundTraffic { tag: String },
    GetStatsOutboundTraffic { tag: String },
    GetStatsSystem,

    CreateUser {
        email: String,
        tags: Option<Vec<String>>,
        inbounds: Option<Vec<String>>,
        traffic_limit: i64,
        reset_traffic_every: Option<i64>,
        expire_at: Option<DateTime<Utc>>,
        ip_limit: i64,
        ip_limit_punishment: Option<IpLimitPunishment>,
        ip_expire_after: i64,
        is_active: bool,
    },
    DeleteUser { email: String },
    GetAllUsers,
}

pub async fn handle_command(pool: PgPool, request: Request) -> anyhow::Result<String> {
    match request {
        Request::StartXray => {
            daemon::start_xray().await?;
            Ok("Xray started".into())
        }
        Request::StopXray => {
            daemon::stop().await?;
            Ok("Xray stopped".into())
        }
        Request::RestartXray => {
            daemon::stop().await?;
            daemon::start_xray().await?;
            Ok("Xray restarted".into())
        }

        Request::GetStatsUserOnlineCount { email } =>
            get_stats_user_online_count(&email).await,
        Request::GetStatsUserOnlineIpList { email } =>
            get_stats_user_online_ip_list(&email).await,
        Request::GetStatsUserTraffic { email } =>
            get_stats_user_traffic(&email).await,
        Request::GetStatsInboundTraffic { tag } =>
            get_stats_inbound_traffic(&tag).await,
        Request::GetStatsOutboundTraffic { tag } =>
            get_stats_outbound_traffic(&tag).await,
        Request::GetStatsSystem =>
            get_stats_system().await,

        Request::CreateUser { email, tags, inbounds,
            traffic_limit, reset_traffic_every, expire_at,
            ip_limit, ip_limit_punishment, ip_expire_after,
            is_active
        } => {
            let ip_limit_punishment = ip_limit_punishment
                .map(|ilp| sqlx::types::Json(ilp));

            let data = CreateUser {
                email,
                tags,
                inbounds,
                traffic_limit,
                reset_traffic_every,
                expire_at,
                ip_limit,
                ip_limit_punishment,
                ip_expire_after,
                is_active,
            };

            let user = crate::data::postgres::create_user(&pool, data).await?;

            create_user(user).await?;

            Ok("User created".to_string())
        }
        Request::DeleteUser { email } => {
            let user = crate::data::postgres::get_user_by_email(
                &pool, &email).await?;

            if user.is_none() {
                bail!("User {} not found", email);
            }

            let user = user.unwrap();

            let successful = crate::data::postgres::delete_user_by_id(
                &pool, user.id).await?;

            remove_user(user).await?;

            Ok(successful.to_string())
        }
        Request::GetAllUsers => {
            let users = crate::data::postgres::get_all_user_emails(
                &pool).await?;

            let formatted = serde_json::to_string_pretty(&users)?;

            Ok(formatted)
        }
    }
}

async fn get_stats_user_online_count(email: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.user_online_count(email).await?;

    Ok(response.to_string())
}

async fn get_stats_user_online_ip_list(email: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.user_online_ip_list(email).await?;

    let formatted = serde_json::to_string_pretty(&response)?;

    Ok(formatted)
}

async fn get_stats_user_traffic(email: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.user_traffic(email).await?;

    let formatted = format!("{} {}", response.0, response.1);

    Ok(formatted)
}

async fn get_stats_inbound_traffic(tag: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.inbound_traffic(tag).await?;

    let formatted = format!("{} {}", response.0, response.1);

    Ok(formatted)
}

async fn get_stats_outbound_traffic(tag: &str) -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.outbound_traffic(tag).await?;

    let formatted = format!("{} {}", response.0, response.1);

    Ok(formatted)
}

async fn get_stats_system() -> anyhow::Result<String> {
    let client = Client::connect().await?;

    let response = client.system_stats().await?;

    let formatted = serde_json::to_string_pretty(
        &SysStatsResponseSerializable::from(response))?;

    Ok(formatted)
}

async fn create_user(
    user: crate::data::postgres::types::User
) -> anyhow::Result<()> {
    let client = Client::connect().await?;

    let id = user.id.to_string();
    let email = user.email;

    // todo there is not only vless exists
    let tags = user.inbounds.unwrap_or(vec![]);
    for tag in tags {
        client.add_vless_user(&tag, &id, &email).await?;
    }

    Ok(())
}

async fn remove_user(
    user: crate::data::postgres::types::User
) -> anyhow::Result<()> {
    let client = Client::connect().await?;

    let email = user.email;

    let tags = user.inbounds.unwrap_or(vec![]);
    for tag in tags {
        client.remove_vless_user(&tag, &email).await?;
    }

    Ok(())
}