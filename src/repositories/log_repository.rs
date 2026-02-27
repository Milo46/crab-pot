use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::query_params::LogQueryParams;
use crate::models::Log;
use crate::repositories::query_builder::LogQueryBuilder;

#[async_trait]
pub trait LogRepositoryTrait {
    async fn get_all_with_cursor(
        &self,
        schema_id: Uuid,
        cursor: Option<i32>,
        limit: i32,
        filters: LogQueryParams,
        forward: bool,
    ) -> AppResult<Vec<Log>>;
    async fn get_by_id(&self, id: i32) -> AppResult<Option<Log>>;
    async fn create(&self, log: &Log) -> AppResult<Log>;
    async fn delete(&self, id: i32) -> AppResult<Option<Log>>;
    async fn delete_all_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64>;

    async fn count_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: Option<&LogQueryParams>,
    ) -> AppResult<i64>;

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
    async fn get_all_with_cursor(
        &self,
        schema_id: Uuid,
        cursor: Option<i32>,
        limit: i32,
        filters: LogQueryParams,
        forward: bool,
    ) -> AppResult<Vec<Log>> {
        let fetch_limit = limit + 1;
        let order = if forward { "DESC" } else { "ASC" };

        let logs = LogQueryBuilder::select()
            .schema_id(schema_id)
            .filters(Some(&filters))
            .cursor(cursor, forward)
            .order_by("created_at", order)
            .then_order_by("id", order)
            .limit(fetch_limit)
            .build()
            .build_query_as::<Log>()
            .fetch_all(&self.pool)
            .await?;

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

    async fn delete(&self, id: i32) -> AppResult<Option<Log>> {
        let deleted_log = sqlx::query_as::<_, Log>(
            "DELETE FROM logs WHERE id = $1 RETURNING id, schema_id, log_data, created_at",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deleted_log)
    }

    async fn count_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: Option<&LogQueryParams>,
    ) -> AppResult<i64> {
        let count: i64 = LogQueryBuilder::count()
            .schema_id(schema_id)
            .filters(query_params)
            .build()
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    async fn delete_all_by_schema_id(&self, schema_id: Uuid) -> AppResult<i64> {
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
