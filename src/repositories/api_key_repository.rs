use base64::{engine::general_purpose, Engine as _};
use rand::{rng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::models::ApiKey;

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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

    pub async fn find_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at,
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_valid_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys
            WHERE key_hash = $1 
                AND is_active = true
                AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        plain_key: &str,
        name: &str,
        description: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        allowed_ips: Option<Vec<std::net::IpAddr>>,
    ) -> Result<ApiKey, sqlx::Error> {
        let key_hash = Self::hash_key(plain_key);
        let key_prefix = Some(format!("{}...", &plain_key[..10]));

        sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (key_hash, key_prefix, name, description, expires_at, allowed_ips)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, key_hash, key_prefix, name, description, created_at, 
                      last_used_at, expires_at, is_active, usage_count, allowed_ips
            "#,
        )
        .bind(&key_hash)
        .bind(&key_prefix)
        .bind(name)
        .bind(description)
        .bind(expires_at)
        .bind(allowed_ips)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_usage(&self, key_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys 
            SET last_used_at = NOW(), 
                usage_count = COALESCE(usage_count, 0) + 1
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn revoke(&self, key_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_expired_active(&self) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys 
            WHERE is_active = true 
                AND expires_at IS NOT NULL 
                AND expires_at <= NOW()
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}
