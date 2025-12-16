use crate::dto::log_dto::{
    LogResponse, PaginatedLogsResponse, PaginationMetadata, QueryParams, TimeWindowMetadata,
};
use crate::error::AppResult;
use crate::models::Log;
use crate::repositories::log_repository::{LogRepository, LogRepositoryTrait};
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
    }

    pub async fn get_log_by_id(&self, id: i32) -> AppResult<Option<Log>> {
        self.log_repository.get_by_id(id).await
    }

    pub async fn create_log(&self, schema_id: Uuid, log_data: Value) -> AppResult<Log> {
        let log = Log {
            id: 0, // This will be set by the database
            schema_id,
            log_data,
            created_at: Utc::now(),
        };

        match self.log_repository.create(&log).await {
            Ok(log) => Ok(log),
            Err(e) => {
                let error_string = e.to_string();
                if error_string.contains("foreign key constraint")
                    || error_string.contains("violates foreign key")
                    || error_string.contains("fk_logs_schema_id")
                    || error_string.contains("resource does not exist")
                {
                    Err(crate::error::AppError::NotFound(format!(
                        "Schema with id '{}' not found",
                        schema_id
                    )))
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn delete_log(&self, id: i32) -> AppResult<bool> {
        self.log_repository.delete(id).await
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
    }

    pub async fn get_paginated_logs(
        &self,
        schema_id: Uuid,
        query_params: QueryParams,
    ) -> AppResult<PaginatedLogsResponse> {
        let logs = self
            .log_repository
            .get_by_schema_id(schema_id, query_params.clone())
            .await?;

        let total = self
            .log_repository
            .count_by_schema_id_with_filters_and_dates(
                schema_id,
                query_params.filters.clone(),
                query_params.date_begin,
                query_params.date_end,
            )
            .await?;

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
