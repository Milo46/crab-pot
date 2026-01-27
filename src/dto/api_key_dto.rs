use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use validator::Validate;

use crate::models::{api_key_model::CreatedApiKey, ApiKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: i32,
    pub key_prefix: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub allowed_ips: Option<Vec<IpNetwork>>,
    pub usage_count: Option<i64>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(value: ApiKey) -> Self {
        Self {
            id: value.id,
            key_prefix: value.key_prefix,
            name: value.name,
            description: value.description,
            created_at: value.created_at,
            last_used_at: value.last_used_at,
            expires_at: value.expires_at,
            is_active: value.is_active,
            allowed_ips: value.allowed_ips,
            usage_count: value.usage_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeysResponse {
    pub api_keys: Vec<ApiKeyResponse>,
}

impl From<Vec<ApiKey>> for ApiKeysResponse {
    fn from(value: Vec<ApiKey>) -> Self {
        Self {
            api_keys: value.into_iter().map(|a| a.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, message = "API key name cannot be empty"))]
    pub name: String,
    pub description: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub allowed_ips: Option<Vec<IpNetwork>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub id: i32,
    pub key: String,
    pub key_prefix: Option<String>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<CreatedApiKey> for CreateApiKeyResponse {
    fn from(value: CreatedApiKey) -> Self {
        Self {
            id: value.api_key.id,
            key: value.plain_key,
            key_prefix: value.api_key.key_prefix,
            name: value.api_key.name,
            created_at: value.api_key.created_at,
            expires_at: value.api_key.expires_at,
        }
    }
}
