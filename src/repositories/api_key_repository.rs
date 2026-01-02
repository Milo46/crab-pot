use sqlx::PgPool;

use crate::{models::ApiKey, AppResult};

pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at,
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_by_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at,
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys
            WHERE key_hash = $1
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn find_valid_by_hash(&self, key_hash: &str) -> AppResult<Option<ApiKey>> {
        let result = sqlx::query_as::<_, ApiKey>(
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
        .await?;

        Ok(result)
    }

    pub async fn create(
        &self,
        key_hash: &str,
        key_prefix: Option<String>,
        name: &str,
        description: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        allowed_ips: Option<Vec<std::net::IpAddr>>,
    ) -> AppResult<ApiKey> {
        let api_key = sqlx::query_as::<_, ApiKey>(
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
                      last_used_at, expires_at, is_active, usage_count, allowed_ips
            "#,
        )
        .bind(key_id)
        .bind(new_key_hash)
        .bind(new_key_prefix)
        .fetch_one(&self.pool)
        .await?;

        Ok(rotated_key)
    }

    pub async fn list(&self) -> AppResult<Vec<ApiKey>> {
        let api_keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, key_hash, key_prefix, name, description, created_at, 
                   last_used_at, expires_at, is_active, usage_count, allowed_ips
            FROM api_keys 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    pub async fn find_expired_active(&self) -> AppResult<Vec<ApiKey>> {
        let expired_active_api_keys = sqlx::query_as::<_, ApiKey>(
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
        .await?;

        Ok(expired_active_api_keys)
    }

    pub async fn delete(&self, id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
