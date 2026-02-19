use sqlx::PgPool;

use crate::{
    models::{api_key_model::NewApiKey, ApiKey},
    AppResult,
};

const API_KEY_COLUMNS: &str = r#"
    id, key_hash, key_prefix, name, description, created_at,
    last_used_at, expires_at, is_active, usage_count, allowed_ips,
    rate_limit_per_second, rate_limit_burst
"#;

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: i32) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(&format!(
            "SELECT {} FROM api_keys WHERE id = $1",
            API_KEY_COLUMNS
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_by_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(&format!(
            "SELECT {} FROM api_keys WHERE key_hash = $1",
            API_KEY_COLUMNS
        ))
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_valid_by_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(&format!(
            "SELECT {}
            FROM api_keys
            WHERE key_hash = $1
                AND is_active = true
                AND (expires_at IS NULL OR expires_at > NOW())",
            API_KEY_COLUMNS
        ))
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn create(&self, new_key: &NewApiKey) -> AppResult<ApiKey> {
        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (key_hash, key_prefix, name, description, expires_at, allowed_ips, rate_limit_per_second, rate_limit_burst)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, key_hash, key_prefix, name, description, created_at, 
                      last_used_at, expires_at, is_active, usage_count, allowed_ips,
                      rate_limit_per_second, rate_limit_burst
            "#,
        )
        .bind(&new_key.key_hash)
        .bind(&new_key.key_prefix)
        .bind(&new_key.name)
        .bind(&new_key.description)
        .bind(new_key.expires_at)
        .bind(&new_key.allowed_ips)
        .bind(new_key.rate_limit_per_second)
        .bind(new_key.rate_limit_burst)
        .fetch_one(&self.pool)
        .await?;

        Ok(api_key)
    }

    pub async fn update_usage(&self, key_hash: &str) -> AppResult<()> {
        let _ = sqlx::query(
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

    pub async fn revoke(&self, key_id: i32) -> AppResult<()> {
        let _ = sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn rotate(
        &self,
        key_id: i32,
        new_key_hash: &str,
        new_key_prefix: Option<String>,
    ) -> AppResult<ApiKey> {
        let rotated_key = sqlx::query_as::<_, ApiKey>(
            r#"
            UPDATE api_keys 
            SET key_hash = $2, 
                key_prefix = $3
            WHERE id = $1
            RETURNING id, key_hash, key_prefix, name, description, created_at, 
                      last_used_at, expires_at, is_active, usage_count, allowed_ips,
                      rate_limit_per_second, rate_limit_burst
            "#,
        )
        .bind(key_id)
        .bind(new_key_hash)
        .bind(new_key_prefix)
        .fetch_one(&self.pool)
        .await?;

        Ok(rotated_key)
    }

    pub async fn get_all(&self) -> AppResult<Vec<ApiKey>> {
        let api_keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips,
                   rate_limit_per_second, rate_limit_burst
            FROM api_keys 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    pub async fn get_expired_active(&self) -> AppResult<Vec<ApiKey>> {
        let expired_active_api_keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips,
                   rate_limit_per_second, rate_limit_burst
            FROM api_keys 
            WHERE is_active = true 
                AND expires_at IS NOT NULL 
                AND expires_at <= NOW()
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(expired_active_api_keys)
    }

    pub async fn delete(&self, id: i32) -> AppResult<Option<ApiKey>> {
        let deleted_api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            DELETE FROM api_keys 
            WHERE id = $1
            RETURNING id, key_hash, key_prefix, name, description, created_at, 
                      last_used_at, expires_at, is_active, usage_count, allowed_ips,
                      rate_limit_per_second, rate_limit_burst
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deleted_api_key)
    }
}
