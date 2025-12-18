use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub key_hash: String,
    pub key_prefix: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub usage_count: Option<i64>,
    #[sqlx(default)]
    pub allowed_ips: Option<Vec<IpNetwork>>,
}

impl ApiKey {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => expires <= Utc::now(),
            None => false,
        }
    }

    pub fn is_ip_allowed(&self, ip: &IpAddr) -> bool {
        match &self.allowed_ips {
            Some(networks) if !networks.is_empty() => {
                networks.iter().any(|network| match (network, ip) {
                    (IpNetwork::V4(net), IpAddr::V4(addr)) => net.contains(*addr),
                    (IpNetwork::V6(net), IpAddr::V6(addr)) => net.contains(*addr),
                    _ => false,
                })
            }
            _ => true,
        }
    }

    pub fn display_key(&self) -> String {
        self.key_prefix.clone().unwrap_or_else(|| "***".to_string())
    }
}

pub struct CreateApiKey {
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<IpNetwork>>,
}
