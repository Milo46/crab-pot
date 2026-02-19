use std::{net::IpAddr, sync::Arc};

use base64::{engine::general_purpose, Engine as _};
use rand::{rng, RngCore};
use sha2::{Digest, Sha256};

use crate::{
    models::{
        api_key_model::{CreatedApiKey, NewApiKey},
        ApiKey, CreateApiKey,
    },
    repositories::ApiKeyRepository,
    AppError, AppResult,
};

#[derive(Clone)]
pub struct ApiKeyService {
    api_key_repository: Arc<ApiKeyRepository>,
}

impl ApiKeyService {
    pub fn new(api_key_repository: Arc<ApiKeyRepository>) -> Self {
        Self { api_key_repository }
    }

    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_key() -> String {
        let mut random_bytes = [0u8; 32];
        rng().fill_bytes(&mut random_bytes);

        format!(
            "sk_{}",
            general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
        )
    }

    pub async fn create_api_key(&self, request: CreateApiKey) -> AppResult<CreatedApiKey> {
        if request.name.trim().is_empty() {
            return Err(AppError::bad_request("API key name cannot be empty"));
        }

        let api_key_plain = Self::generate_key();

        let api_key_hash = Self::hash_key(&api_key_plain);
        let api_key_prefix = Some(format!("{}...", &api_key_plain[..10]));
        let allowed_ips: Option<Vec<IpAddr>> = request
            .allowed_ips
            .map(|networks| networks.into_iter().map(|net| net.ip()).collect());

        let new_key = NewApiKey {
            key_hash: api_key_hash,
            key_prefix: api_key_prefix,
            name: request.name,
            description: request.description,
            expires_at: request.expires_at,
            allowed_ips,
            rate_limit_per_second: request.rate_limit_per_second,
            rate_limit_burst: request.rate_limit_burst,
        };

        let api_key = self
            .api_key_repository
            .create(&new_key)
            .await
            .map_err(|e| e.context("Failed to create API key"))?;

        Ok(CreatedApiKey {
            api_key,
            plain_key: api_key_plain,
        })
    }

    pub async fn rotate_api_key(&self, id: i32) -> AppResult<CreatedApiKey> {
        let _ = self.find_by_id(id).await?;

        let new_plain_key = Self::generate_key();
        let new_key_hash = Self::hash_key(&new_plain_key);
        let new_key_prefix = Some(format!("{}...", &new_plain_key[..10]));

        let rotated_key = self
            .api_key_repository
            .rotate(id, &new_key_hash, new_key_prefix)
            .await
            .map_err(|e| e.context(format!("Failed to rotate API key {}", id)))?;

        Ok(CreatedApiKey {
            api_key: rotated_key,
            plain_key: new_plain_key,
        })
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<ApiKey> {
        self.api_key_repository
            .get_by_id(id)
            .await
            .map_err(|e| e.context(format!("Failed to fetch API key {}", id)))?
            .ok_or_else(|| AppError::not_found(format!("Api key with id {} not found", id)))
    }

    pub async fn find_valid_by_hash(&self, key_hash: &str) -> AppResult<ApiKey> {
        self.api_key_repository
            .get_valid_by_hash(key_hash)
            .await
            .map_err(|e| e.context("Failed to validate API key"))?
            .ok_or_else(|| AppError::not_found("Valid API key not found".to_string()))
    }

    pub async fn update_usage(&self, key_hash: &str) -> AppResult<()> {
        self.api_key_repository
            .update_usage(key_hash)
            .await
            .map_err(|e| e.context("Failed to update API key usage"))
    }

    pub async fn list_api_keys(&self) -> AppResult<Vec<ApiKey>> {
        self.api_key_repository
            .get_all()
            .await
            .map_err(|e| e.context("Failed to list API keys"))
    }

    pub async fn delete_api_key(&self, id: i32) -> AppResult<ApiKey> {
        self.api_key_repository
            .delete(id)
            .await
            .map_err(|e| e.context(format!("Failed to delete API key {}", id)))?
            .ok_or_else(|| AppError::not_found(format!("API key with id {} not found", id)))
    }
}
