use std::collections::HashMap;
use xray_core::{
    app::{
        log::command::logger_service_client::LoggerServiceClient,
        proxyman::command::handler_service_client::HandlerServiceClient,
        router::command::routing_service_client::RoutingServiceClient,
        stats::command::stats_service_client::StatsServiceClient,
    },
    core::observatory::command::observatory_service_client::ObservatoryServiceClient,
    transport::internet::grpc::grpc_service_client::GrpcServiceClient,
};
use std::env;
use tonic::transport::{Channel, Endpoint};
use crate::xray_core::app::stats::command::GetStatsRequest;

pub mod xray_core;

pub async fn connect() -> anyhow::Result<Channel> {
    let channel = Endpoint::try_from(
        format!(
            "https://127.0.0.1:{}",
            env::var("XRAY_API_PORT")
                .map_err(|_| anyhow::anyhow!("XRAY_API_PORT not found"))?
        ))?
        .connect()
        .await?;

    Ok(channel)
}

pub struct Client {
    channel: Channel,
}

impl Client {
    pub async fn connect() -> anyhow::Result<Self> {
        let channel = connect().await?;
        Ok(Self { channel })
    }

    #[inline]
    pub fn logger(&self) -> LoggerServiceClient<Channel> {
        LoggerServiceClient::new(self.channel.clone())
    }

    #[inline]
    pub fn handler(&self) -> HandlerServiceClient<Channel> {
        HandlerServiceClient::new(self.channel.clone())
    }

    #[inline]
    pub fn routing(&self) -> RoutingServiceClient<Channel> {
        RoutingServiceClient::new(self.channel.clone())
    }

    #[inline]
    pub fn stats(&self) -> StatsServiceClient<Channel> {
        StatsServiceClient::new(self.channel.clone())
    }

    #[inline]
    pub fn observatory(&self) -> ObservatoryServiceClient<Channel> {
        ObservatoryServiceClient::new(self.channel.clone())
    }

    #[inline]
    pub fn grpc(&self) -> GrpcServiceClient<Channel> {
        GrpcServiceClient::new(self.channel.clone())
    }

    pub async fn user_traffic(
        &self,
        email: &str,
    ) -> anyhow::Result<(i64, i64)> {
        let mut client = self.stats();

        let up_name = format!("user>>>{}>>>traffic>>>uplink", email);
        let down_name = format!("user>>>{}>>>traffic>>>downlink", email);

        let up = client
            .get_stats(GetStatsRequest { name: up_name, reset: false })
            .await?
            .into_inner()
            .stat
            .map(|s| s.value)
            .unwrap_or(0);

        let down = client
            .get_stats(GetStatsRequest { name: down_name, reset: false })
            .await?
            .into_inner()
            .stat
            .map(|s| s.value)
            .unwrap_or(0);

        Ok((up, down))
    }

    pub async fn user_online_ip_list(
        &self,
        email: &str,
    ) -> anyhow::Result<HashMap<String, i64>> {
        let mut client = self.stats();

        let name = format!("user>>>{}>>>online", email);

        let resp = client
            .get_stats_online_ip_list(GetStatsRequest { name, reset: false })
            .await?
            .into_inner();

        Ok(resp.ips)
    }
}