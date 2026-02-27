use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};

use crate::dto::api_key_dto::CreateApiKeyRequest;

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
    pub rate_limit_per_second: Option<i32>,
    pub rate_limit_burst: Option<i32>,
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

#[derive(Debug, Clone)]
pub struct CreateApiKey {
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<IpNetwork>>,
    pub rate_limit_per_second: Option<i32>,
    pub rate_limit_burst: Option<i32>,
}

impl CreateApiKey {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            expires_at: None,
            allowed_ips: None,
            rate_limit_per_second: None,
            rate_limit_burst: None,
        }
    }
}

impl From<CreateApiKeyRequest> for CreateApiKey {
    fn from(value: CreateApiKeyRequest) -> Self {
        CreateApiKey {
            name: value.name,
            description: value.description,
            expires_at: value.expires_at,
            allowed_ips: value.allowed_ips,
            rate_limit_per_second: value.rate_limit_per_second,
            rate_limit_burst: value.rate_limit_burst,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedApiKey {
    #[serde(flatten)]
    pub api_key: ApiKey,
    pub plain_key: String,
}

pub struct NewApiKey {
    pub key_hash: String,
    pub key_prefix: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<IpAddr>>,
    pub rate_limit_per_second: Option<i32>,
    pub rate_limit_burst: Option<i32>,
}
