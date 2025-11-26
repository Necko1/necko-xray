use serde::{Deserialize, Serialize};
use necko_xray::Client;
use necko_xray::xray_core::app::stats::command::SysStatsResponseSerializable;

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
}

pub async fn handle_command(request: Request) -> anyhow::Result<String> {
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