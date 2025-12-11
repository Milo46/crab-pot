use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    dto::{
        log_dto::{TimeWindowMetadata, QueryParams}, CreateLogRequest, ErrorResponse, LogEvent, LogResponse,
        PaginatedLogsResponse, PaginationMetadata, QueryLogsRequest,
    },
    AppState,
};

pub async fn get_logs_default(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Query(params): Query<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_logs(
        State(state),
        Path((schema_name, "1.0.0".to_string())),
        Query(params),
    )
    .await
}

pub async fn get_logs(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Query(payload): Query<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
            )),
        ));
    }

    let query_params: QueryParams = payload.into();

    match state
        .log_service
        .get_logs_by_schema_name_and_id(
            &schema_name,
            &schema_version,
            query_params.clone(),
        )
        .await
    {
        Ok(logs) => {
            let total = state
                .log_service
                .count_logs_by_schema_name_and_id_with_dates(
                    &schema_name,
                    &schema_version,
                    query_params.filters.clone(),
                    query_params.date_begin,
                    query_params.date_end,
                )
                .await
                .unwrap_or(0);

            let log_responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

            let total_pages = if query_params.limit > 0 {
                ((total as f64) / (query_params.limit as f64)).ceil() as i32
            } else {
                0
            };

            Ok(Json(PaginatedLogsResponse {
                logs: log_responses,
                timewindow: if query_params.date_begin.is_some() || query_params.date_end.is_some() {
                    Some(TimeWindowMetadata {
                        date_begin: query_params.date_begin,
                        date_end: query_params.date_end,
                    })
                } else {
                    None
                },
                pagination: PaginationMetadata {
                    page: query_params.page,
                    limit: query_params.limit,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("NOT_FOUND", e.to_string())),
            ))
        }
    }
}

pub async fn query_logs(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Json(payload): Json<QueryLogsRequest>,
) -> Result<Json<PaginatedLogsResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
            )),
        ));
    }

    let query_params: QueryParams = payload.into();

    match state
        .log_service
        .get_logs_by_schema_name_and_id(
            &schema_name,
            &schema_version,
            query_params.clone(),
        )
        .await
    {
        Ok(logs) => {
            let total = state
                .log_service
                .count_logs_by_schema_name_and_id_with_dates(
                    &schema_name,
                    &schema_version,
                    query_params.filters.clone(),
                    query_params.date_begin,
                    query_params.date_end,
                )
                .await
                .unwrap_or(0);

            let log_responses: Vec<LogResponse> = logs.into_iter().map(LogResponse::from).collect();

            let total_pages = if query_params.limit > 0 {
                ((total as f64) / (query_params.limit as f64)).ceil() as i32
            } else {
                0
            };

            Ok(Json(PaginatedLogsResponse {
                logs: log_responses,
                timewindow: if query_params.date_begin.is_some() || query_params.date_end.is_some() {
                    Some(TimeWindowMetadata {
                        date_begin: query_params.date_begin,
                        date_end: query_params.date_end,
                    })
                } else {
                    None
                },
                pagination: PaginationMetadata {
                    page: query_params.page,
                    limit: query_params.limit,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            let status_code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status_code,
                Json(ErrorResponse::new("NOT_FOUND", e.to_string())),
            ))
        }
    }
}

pub async fn get_log_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<LogResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.log_service.get_log_by_id(id).await {
        Ok(Some(log)) => Ok(Json(LogResponse::from(log))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Log with id '{}' not found", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("FETCH_FAILED", e.to_string())),
        )),
    }
}

pub async fn create_log(
    State(state): State<AppState>,
    Json(payload): Json<CreateLogRequest>,
) -> Result<(StatusCode, Json<LogResponse>), (StatusCode, Json<ErrorResponse>)> {
    if payload.schema_id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
            )),
        ));
    }

    if !payload.log_data.is_object() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Log data must be a JSON object",
            )),
        ));
    }

    match state
        .log_service
        .create_log(payload.schema_id, payload.log_data)
        .await
    {
        Ok(log) => {
            let _ = state
                .log_broadcast
                .send(LogEvent::created_from(log.clone()));
            Ok((StatusCode::CREATED, Json(LogResponse::from(log))))
        }
        Err(e) => {
            let (status_code, error) = if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "NOT_FOUND")
            } else if e.to_string().contains("validation")
                || e.to_string().contains("Required field")
            {
                (StatusCode::BAD_REQUEST, "VALIDATION_FAILED")
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR")
            };

            Err((status_code, Json(ErrorResponse::new(error, e.to_string()))))
        }
    }
}

pub async fn delete_log(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let log = state.log_service.get_log_by_id(id).await;
    match state.log_service.delete_log(id).await {
        Ok(true) => {
            if let Ok(Some(log)) = log {
                let _ = state.log_broadcast.send(LogEvent::deleted_from(log));
            }
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Log with id '{}' not found", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("DELETION_FAILED", e.to_string())),
        )),
    }
}
