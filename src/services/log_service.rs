use crate::dto::CursorMetadata;
use crate::error::AppResult;
use crate::models::query_params::LogQueryParams;
use crate::models::Log;
use crate::repositories::log_repository::{LogRepository, LogRepositoryTrait};
use crate::repositories::schema_repository::{SchemaRepository, SchemaRepositoryTrait};
use crate::AppError;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogService {
    log_repository: Arc<LogRepository>,
    schema_repository: Arc<SchemaRepository>,
}

impl LogService {
    pub fn new(
        log_repository: Arc<LogRepository>,
        schema_repository: Arc<SchemaRepository>,
    ) -> Self {
        Self {
            log_repository,
            schema_repository,
        }
    }

    pub async fn get_log_by_id(&self, id: i32) -> AppResult<Log> {
        self.log_repository
            .get_by_id(id)
            .await
            .map_err(|e| e.context(format!("Failed to fetch log {}", id)))?
            .ok_or_else(|| AppError::not_found(format!("Log with id {} not found", id)))
    }

    pub async fn create_log(&self, schema_id: Uuid, log_data: Value) -> AppResult<Log> {
        if schema_id.is_nil() {
            return Err(AppError::bad_request("Schema ID cannot be empty"));
        }

        if !log_data.is_object() {
            return Err(AppError::bad_request("Log data must be a JSON object"));
        }

        let log = Log {
            id: 0, // This will be set by the database
            schema_id,
            log_data,
            created_at: Utc::now(),
        };

        self.log_repository
            .create(&log)
            .await
            .map_err(|e| e.context(format!("Failed to create log for schema {}", schema_id)))
    }

    pub async fn delete_log(&self, id: i32) -> AppResult<Log> {
        self.log_repository
            .delete(id)
            .await
            .map_err(|e| e.context(format!("Failed to delete log {}", id)))?
            .ok_or_else(|| AppError::not_found(format!("Log with id {} not found", id)))
    }

    pub async fn count_logs_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: &LogQueryParams,
    ) -> AppResult<i64> {
        self.log_repository
            .count_by_schema_id(schema_id, Some(query_params))
            .await
            .map_err(|e| e.context(format!("Failed to count logs for schema {}", schema_id)))
    }

    pub async fn get_cursor_logs(
        &self,
        schema_id: Uuid,
        cursor: Option<i32>,
        limit: i32,
        filters: LogQueryParams,
    ) -> AppResult<(Vec<Log>, CursorMetadata<i32>)> {
        if schema_id.is_nil() {
            return Err(AppError::bad_request("Schema ID cannot be nil"));
        }

        if limit <= 0 {
            return Err(AppError::bad_request("Limit must be greater than 0"));
        }

        let schema_exists = self
            .schema_repository
            .get_by_id(schema_id)
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to check schema existence for {}",
                    schema_id
                ))
            })?;

        if schema_exists.is_none() {
            return Err(AppError::not_found(format!(
                "Schema with id {} not found",
                schema_id
            )));
        }

        let mut logs = self
            .log_repository
            .get_all_with_cursor(schema_id, cursor, limit, filters)
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to get logs with cursor feature for schema {}",
                    schema_id
                ))
            })?;

        let has_more = logs.len() > limit as usize;

        if has_more {
            logs.pop();
        }

        let next_cursor = if has_more {
            logs.last().map(|log| log.id)
        } else {
            None
        };

        // For backward pagination (toward newer logs):
        // prev_cursor would be first_id + 1, but we don't support
        // bidirectional pagination yet. Setting to None for clarity.
        let prev_cursor = None;

        Ok((
            logs,
            CursorMetadata::<i32> {
                limit,
                next_cursor,
                prev_cursor,
                has_more,
            },
        ))
    }

    pub async fn get_initial_cursor(&self, schema_id: Uuid) -> AppResult<i32> {
        if schema_id.is_nil() {
            return Err(AppError::bad_request("Schema ID cannot be nil"));
        }

        let latest_id = self
            .log_repository
            .get_latest_log_id(schema_id)
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to get latest log ID for schema {}",
                    schema_id
                ))
            })?;

        Ok(latest_id.map(|id| id + 1).unwrap_or(i32::MAX))
    }
}
