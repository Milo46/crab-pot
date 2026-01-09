use crate::dto::log_dto::{
    LogResponse, PaginatedLogsResponse, PaginationMetadata, TimeWindowMetadata,
};
use crate::error::AppResult;
use crate::models::{Log, QueryParams};
use crate::repositories::log_repository::{LogRepository, LogRepositoryTrait};
use crate::AppError;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogService {
    log_repository: Arc<LogRepository>,
}

impl LogService {
    pub fn new(log_repository: Arc<LogRepository>) -> Self {
        Self { log_repository }
    }

    pub async fn get_logs_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<Vec<Log>> {
        self.log_repository
            .get_by_schema_id(schema_id, query_params)
            .await
            .map_err(|e| e.context(format!("Failed to get logs for schema {}", schema_id)))
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
            .map_err(|e| e.context(format!("Failed to delete log {}", id)))
    }

    pub async fn count_logs_by_schema_id(
        &self,
        schema_id: Uuid,
        query_params: &QueryParams,
    ) -> AppResult<i64> {
        self.log_repository
            .count_by_schema_id_with_filters_and_dates(
                schema_id,
                query_params.filters.clone(),
                query_params.date_begin,
                query_params.date_end,
            )
            .await
            .map_err(|e| e.context(format!("Failed to count logs for schema {}", schema_id)))
    }

    pub async fn get_paginated_logs(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<PaginatedLogsResponse> {
        if schema_id.is_nil() {
            return Err(AppError::bad_request("Schema ID cannot be nil"));
        }

        let logs = self
            .log_repository
            .get_by_schema_id(schema_id, query_params.clone())
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to get paginated logs for schema {}",
                    schema_id
                ))
            })?;

        let total = self
            .log_repository
            .count_by_schema_id_with_filters_and_dates(
                schema_id,
                query_params.filters.clone(),
                query_params.date_begin,
                query_params.date_end,
            )
            .await
            .map_err(|e| e.context(format!("Failed to count logs for schema {}", schema_id)))?;

        let log_responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

        let total_pages = if query_params.limit > 0 {
            ((total as f64) / (query_params.limit as f64)).ceil() as i32
        } else {
            0
        };

        let timewindow = if query_params.date_begin.is_some() || query_params.date_end.is_some() {
            Some(TimeWindowMetadata {
                date_begin: query_params.date_begin,
                date_end: query_params.date_end,
            })
        } else {
            None
        };

        Ok(PaginatedLogsResponse {
            logs: log_responses,
            timewindow,
            pagination: PaginationMetadata {
                page: query_params.page,
                limit: query_params.limit,
                total,
                total_pages,
            },
        })
    }
}
