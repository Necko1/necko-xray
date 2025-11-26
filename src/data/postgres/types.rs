use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum IpLimitPunishment {
    Nothing,
    SuspendUser { time: i64 },
    BanLastIp { time: i64 },
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub tags: Option<Vec<String>>,
    /// Inbounds to add user to
    pub inbounds: Option<Vec<String>>,
    /// Traffic limit in bytes
    pub traffic_limit: i64,
    /// Traffic used in bytes
    pub traffic_used: i64,
    /// Reset traffic_used every X seconds
    pub reset_traffic_every: Option<i64>,
    pub last_traffic_reset_at: Option<DateTime<Utc>>,
    pub expire_at: Option<DateTime<Utc>>,
    /// 0 = no limit
    pub ip_limit: i64,
    pub ip_list: Option<Json<HashMap<String, i64>>>,
    /// What to do when user exceeds ip_limit
    pub ip_limit_punishment: Option<Json<IpLimitPunishment>>,
    /// Expire IP from ip_list after X seconds (0 = never)
    pub ip_expire_after: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub tags: Option<Vec<String>>,
    /// Inbounds to add user to
    pub inbounds: Option<Vec<String>>,
    /// Traffic limit in bytes
    pub traffic_limit: i64,
    /// Reset traffic_used every X seconds
    pub reset_traffic_every: Option<i64>,
    pub expire_at: Option<DateTime<Utc>>,
    /// 0 = no limit
    pub ip_limit: i64,
    /// What to do when user exceeds ip_limit
    pub ip_limit_punishment: Option<Json<IpLimitPunishment>>,
    /// Expire IP from ip_list after X seconds (0 = never)
    pub ip_expire_after: i64,
    pub is_active: bool,
}

impl Default for CreateUser {
    fn default() -> Self {
        Self {
            email: String::new(),
            tags: None,
            inbounds: None,
            traffic_limit: 0,
            reset_traffic_every: None,
            expire_at: None,
            ip_limit: 0,
            ip_limit_punishment: None,
            ip_expire_after: 0,
            is_active: true,
        }
    }
}