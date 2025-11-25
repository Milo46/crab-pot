use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::Log;

#[async_trait]
pub trait LogRepositoryTrait {
    async fn get_by_schema_id(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
    ) -> AppResult<Vec<Log>>;
    async fn get_by_id(&self, id: i32) -> AppResult<Option<Log>>;
    async fn create(&self, log: &Log) -> AppResult<Log>;
    async fn delete(&self, id: i32) -> AppResult<bool>;
    async fn count_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64>;
    async fn delete_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64>;
}

#[derive(Clone)]
pub struct LogRepository {
    pool: PgPool,
}

impl LogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LogRepositoryTrait for LogRepository {
    async fn get_by_schema_id(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
    ) -> AppResult<Vec<Log>> {
        if let Some(filter_obj) = &filters {
            if let Some(filter_map) = filter_obj.as_object() {
                let logs = sqlx::query_as::<_, Log>(
                    "SELECT * FROM logs WHERE schema_id = $1 AND log_data @> $2 ORDER BY created_at DESC"
                )
                .bind(schema_id)
                .bind(filter_obj)
                .fetch_all(&self.pool)
                .await?;

                tracing::debug!(
                    "Fetched {} logs for schema_id={} with filters: {:?}",
                    logs.len(),
                    schema_id,
                    filter_map.keys().collect::<Vec<_>>()
                );

                return Ok(logs);
            }
        }

        let logs = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs WHERE schema_id = $1 ORDER BY created_at DESC",
        )
        .bind(schema_id)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!(
            "Fetched {} logs for schema_id={} (no filters)",
            logs.len(),
            schema_id
        );

        Ok(logs)
    }

    async fn get_by_id(&self, id: i32) -> AppResult<Option<Log>> {
        let log = sqlx::query_as::<_, Log>("SELECT * FROM logs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(log)
    }

    async fn create(&self, log: &Log) -> AppResult<Log> {
        let created_log = sqlx::query_as::<_, Log>(
            r#"
            INSERT INTO logs (schema_id, log_data, created_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(log.schema_id)
        .bind(&log.log_data)
        .bind(log.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(created_log)
    }

    async fn delete(&self, id: i32) -> AppResult<bool> {
        let result = sqlx::query("DELETE FROM logs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn count_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM logs WHERE schema_id = $1")
            .bind(schema_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn delete_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64> {
        let result = sqlx::query("DELETE FROM logs WHERE schema_id = $1")
            .bind(schema_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }
}
