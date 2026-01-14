use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Log;
use crate::repositories::query_builder::LogQueryBuilder;
use crate::QueryParams;

#[async_trait]
pub trait LogRepositoryTrait {
    async fn get_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<Vec<Log>>;
    async fn get_by_id(&self, id: i32) -> AppResult<Option<Log>>;
    async fn create(&self, log: &Log) -> AppResult<Log>;
    async fn delete(&self, id: i32) -> AppResult<Log>;
    async fn count_by_schema_id(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
        date_begin: Option<DateTime<Utc>>,
        date_end: Option<DateTime<Utc>>,
    ) -> AppResult<i64>;
    async fn delete_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64>;

    async fn get_by_schema_id_with_cursor(
        &self,
        schema_id: Uuid,
        cursor: i32,
        limit: i32,
    ) -> AppResult<Vec<Log>>;
    
    async fn get_latest_log_id(&self, schema_id: Uuid) -> AppResult<Option<i32>>;
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
    async fn get_by_schema_id_with_cursor(
        &self,
        schema_id: Uuid,
        cursor: i32,
        limit: i32,
    ) -> AppResult<Vec<Log>> {
        let fetch_limit = limit + 1;
        
        let logs = sqlx::query_as::<_, Log>(
            r#"
            SELECT * FROM logs
            WHERE schema_id = $1 AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(schema_id)
        .bind(cursor)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    async fn get_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<Vec<Log>> {
        let logs = LogQueryBuilder::select()
            .schema_id(schema_id)
            .filters(query_params.filters.as_ref())
            .date_range(query_params.date_begin, query_params.date_end)
            .order_by("created_at", "DESC")
            .paginate(query_params.page, query_params.limit)
            .build()
            .build_query_as::<Log>()
            .fetch_all(&self.pool)
            .await?;

        tracing::debug!(
            "Fetched {} logs for schema_id={} (filters: {}, dates: {})",
            logs.len(),
            schema_id,
            query_params.filters.is_some(),
            query_params.date_begin.is_some() && query_params.date_end.is_some()
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

    async fn delete(&self, id: i32) -> AppResult<Log> {
        let deleted_log = sqlx::query_as::<_, Log>(
            "DELETE FROM logs WHERE id = $1 RETURNING id, schema_id, log_data, created_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Log with id '{}' not found", id)))?;

        Ok(deleted_log)
    }

    async fn count_by_schema_id(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
        date_begin: Option<DateTime<Utc>>,
        date_end: Option<DateTime<Utc>>,
    ) -> AppResult<i64> {
        let count: i64 = LogQueryBuilder::count()
            .schema_id(schema_id)
            .filters(filters.as_ref())
            .date_range(date_begin, date_end)
            .build()
            .build_query_scalar()
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
    
    async fn get_latest_log_id(&self, schema_id: Uuid) -> AppResult<Option<i32>> {
        let result = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT id FROM logs
            WHERE schema_id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
        )
        .bind(schema_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }
}
