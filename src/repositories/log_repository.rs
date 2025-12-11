use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::Log;
use crate::dto::log_dto::QueryParams;

#[async_trait]
pub trait LogRepositoryTrait {
    async fn get_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<Vec<Log>>;
    async fn get_by_id(&self, id: i32) -> AppResult<Option<Log>>;
    async fn create(&self, log: &Log) -> AppResult<Log>;
    async fn delete(&self, id: i32) -> AppResult<bool>;
    async fn count_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64>;
    async fn count_by_schema_id_with_filters(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
    ) -> AppResult<i64>;
    async fn count_by_schema_id_with_filters_and_dates(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
        date_begin: Option<DateTime<Utc>>,
        date_end: Option<DateTime<Utc>>,
    ) -> AppResult<i64>;
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
        query_params: QueryParams,
    ) -> AppResult<Vec<Log>> {
        let offset = (query_params.page - 1) * query_params.limit;
        let has_filters = query_params.filters.as_ref().and_then(|f| f.as_object()).is_some();
        let has_dates = query_params.date_begin.is_some() && query_params.date_end.is_some();

        let logs = match (has_filters, has_dates) {
            (true, true) => {
                sqlx::query_as::<_, Log>(
                    r#"
                    SELECT * FROM logs
                    WHERE schema_id = $1 AND log_data @> $2 AND created_at BETWEEN $3 AND $4
                    ORDER BY created_at DESC
                    LIMIT $5 OFFSET $6
                    "#,
                )
                .bind(schema_id)
                .bind(&query_params.filters)
                .bind(query_params.date_begin)
                .bind(query_params.date_end)
                .bind(query_params.limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (true, false) => {
                sqlx::query_as::<_, Log>(
                    r#"
                    SELECT * FROM logs
                    WHERE schema_id = $1 AND log_data @> $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(schema_id)
                .bind(&query_params.filters)
                .bind(query_params.limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (false, true) => {
                sqlx::query_as::<_, Log>(
                    r#"
                    SELECT * FROM logs
                    WHERE schema_id = $1 AND created_at BETWEEN $2 AND $3
                    ORDER BY created_at DESC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(schema_id)
                .bind(query_params.date_begin)
                .bind(query_params.date_end)
                .bind(query_params.limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (false, false) => {
                sqlx::query_as::<_, Log>(
                    r#"
                    SELECT * FROM logs
                    WHERE schema_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(schema_id)
                .bind(query_params.limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        tracing::debug!(
            "Fetched {} logs for schema_id={} (filters: {}, dates: {})",
            logs.len(),
            schema_id,
            has_filters,
            has_dates
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

    async fn count_by_schema_id_with_filters(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
    ) -> AppResult<i64> {
        if let Some(filter_obj) = &filters {
            if filter_obj.as_object().is_some() {
                let count = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM logs WHERE schema_id = $1 AND log_data @> $2",
                )
                .bind(schema_id)
                .bind(filter_obj)
                .fetch_one(&self.pool)
                .await?;

                return Ok(count);
            }
        }

        self.count_by_schema_id(schema_id).await
    }

    async fn count_by_schema_id_with_filters_and_dates(
        &self,
        schema_id: Uuid,
        filters: Option<Value>,
        date_begin: Option<DateTime<Utc>>,
        date_end: Option<DateTime<Utc>>,
    ) -> AppResult<i64> {
        match (date_begin, date_end) {
            (Some(begin), Some(end)) => {
                // Both dates provided
                if let Some(filter_obj) = &filters {
                    if filter_obj.as_object().is_some() {
                        let count = sqlx::query_scalar::<_, i64>(
                            r#"
                            SELECT COUNT(*) FROM logs
                            WHERE
                                schema_id = $1 AND
                                log_data @> $2 AND
                                created_at BETWEEN $3 AND $4
                            "#,
                        )
                        .bind(schema_id)
                        .bind(filter_obj)
                        .bind(begin)
                        .bind(end)
                        .fetch_one(&self.pool)
                        .await?;

                        return Ok(count);
                    }
                }

                let count = sqlx::query_scalar::<_, i64>(
                    r#"
                    SELECT COUNT(*) FROM logs
                    WHERE
                        schema_id = $1 AND
                        created_at BETWEEN $2 AND $3
                    "#,
                )
                .bind(schema_id)
                .bind(begin)
                .bind(end)
                .fetch_one(&self.pool)
                .await?;

                Ok(count)
            }
            _ => {
                // Dates not provided or only one provided - count without date filtering
                if let Some(filter_obj) = &filters {
                    if filter_obj.as_object().is_some() {
                        let count = sqlx::query_scalar::<_, i64>(
                            r#"
                            SELECT COUNT(*) FROM logs
                            WHERE
                                schema_id = $1 AND
                                log_data @> $2
                            "#,
                        )
                        .bind(schema_id)
                        .bind(filter_obj)
                        .fetch_one(&self.pool)
                        .await?;

                        return Ok(count);
                    }
                }

                self.count_by_schema_id(schema_id).await
            }
        }
    }

    async fn delete_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64> {
        let result = sqlx::query("DELETE FROM logs WHERE schema_id = $1")
            .bind(schema_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }
}
