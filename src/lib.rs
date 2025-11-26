use crate::proto::app::proxyman::command::{AddUserOperation, AlterInboundRequest, RemoveUserOperation};
use crate::proto::app::stats::command::{GetStatsRequest, SysStatsRequest, SysStatsResponse};
use crate::proto::common::protocol::User;
use crate::proto::common::serial;
use crate::proto::proxy::vless::Account as VlessAccount;
use proto::{
    app::{
        log::command::logger_service_client::LoggerServiceClient,
        proxyman::command::handler_service_client::HandlerServiceClient,
        router::command::routing_service_client::RoutingServiceClient,
        stats::command::stats_service_client::StatsServiceClient,
    },
    core::observatory::command::observatory_service_client::ObservatoryServiceClient,
    transport::internet::grpc::grpc_service_client::GrpcServiceClient,
};
use std::collections::HashMap;
use std::env;
use tonic::transport::{Channel, Endpoint};

pub mod proto;
pub mod core;
pub mod api;
pub mod config;
pub mod data;
pub mod datetime;

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
}

impl Client {
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

    pub async fn user_online_count(&self, email: &str) -> anyhow::Result<i64> {
        let mut client = self.stats();

        let name = format!("user>>>{}>>>online", email);

        let val = client
            .get_stats_online(GetStatsRequest { name, reset: false })
            .await?
            .into_inner()
            .stat
            .map(|s| s.value)
            .unwrap_or(0);

        Ok(val)
    }

    pub async fn system_stats(&self) -> anyhow::Result<SysStatsResponse> {
        let mut client = self.stats();

        let val = client
            .get_sys_stats(SysStatsRequest {})
            .await?
            .into_inner();

        Ok(val)
    }
    pub async fn user_traffic(
        &self,
        email: &str,
    ) -> anyhow::Result<(i64, i64)> {
        self.some_traffic("user", email).await
    }

    pub async fn inbound_traffic(
        &self,
        tag: &str
    ) -> anyhow::Result<(i64, i64)> {
        self.some_traffic("inbound", tag).await
    }

    pub async fn outbound_traffic(
        &self,
        tag: &str
    ) -> anyhow::Result<(i64, i64)> {
        self.some_traffic("outbound", tag).await
    }

    async fn some_traffic(
        &self,
        traffic_from: &str,
        r#for: &str
    ) -> anyhow::Result<(i64, i64)> {
        let mut client = self.stats();

        let up_name = format!("{}>>>{}>>>traffic>>>uplink", traffic_from, r#for);
        let down_name = format!("{}>>>{}>>>traffic>>>downlink", traffic_from, r#for);

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
}

impl Client {
    pub async fn add_vless_user(
        &self,
        inbound_tag: &str,
        id: &str,
        email: &str
    ) -> anyhow::Result<()> {
        let mut client = self.handler();

        let account = VlessAccount {
            id: id.to_string(),
            flow: "".to_string(),
            encryption: "none".to_string(),
            ..Default::default()
        };

        let inbound_user = User {
            level: 0,
            email: email.to_string(),
            account: Some(serial::to_typed_message(&account, "xray.proxy.vless.Account")),
        };

        let op = AddUserOperation { user: Some(inbound_user) };

        let req = AlterInboundRequest {
            tag: inbound_tag.to_string(),
            operation: Some(serial::to_typed_message(
                &op,
                "xray.proxyman.command.AddUserOperation"
            )),
        };

        let _ = client.alter_inbound(req).await?;
        Ok(())
    }

    pub async fn remove_vless_user(
        &self,
        inbound_tag: &str,
        email: &str
    ) -> anyhow::Result<()> {
        let mut client = self.handler();

        let op = RemoveUserOperation { email: email.to_string() };

        let req = AlterInboundRequest {
            tag: inbound_tag.to_string(),
            operation: Some(serial::to_typed_message(
                &op,
                "xray.proxyman.command.RemoveUserOperation"
            )),
        };

        let _ = client.alter_inbound(req).await?;
        Ok(())
    }
}