use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
// use crate::models::log_model::Log;

use crate::models::Log;

use anyhow::Result;

#[async_trait]
pub trait LogRepositoryTrait {
    async fn get_by_schema_id(&self, schema_id: Uuid) -> Result<Vec<Log>>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Log>>;
    async fn create(&self, log: &Log) -> Result<Log>;
    async fn delete(&self, id: i32) -> Result<bool>;
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
    async fn get_by_schema_id(&self, schema_id: Uuid) -> Result<Vec<Log>> {
        let logs = sqlx::query_as::<_, Log>(
            "SELECT * FROM logs WHERE schema_id = $1 ORDER BY created_at DESC"
        )
        .bind(schema_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Log>> {
        let log = sqlx::query_as::<_, Log>("SELECT * FROM logs WHERE id = $1")
           .bind(id)
           .fetch_optional(&self.pool)
           .await?;

        // let log = sqlx::query_as!(
        //     Log,
        //     "SELECT id, schema_id, log_data, created_at FROM logs WHERE id = $1",
        //     id
        // )
        //     .fetch_optional(&self.pool)
        //     .await?;

        Ok(log)
    }

    async fn create(&self, log: &Log) -> Result<Log> {
        let created_log = sqlx::query_as::<_, Log>(
            r#"
            INSERT INTO logs (schema_id, log_data, created_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#
        )
        .bind(&log.schema_id)
        .bind(&log.log_data)
        .bind(&log.created_at)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(created_log)
    }

    async fn delete(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM logs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
