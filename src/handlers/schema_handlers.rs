use axum::{
    Extension, Json, extract::{Path, Query, State}, http::{HeaderMap, StatusCode, header}, response::IntoResponse
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    AppState, dto::{
        CreateSchemaRequest, DeleteSchemaQuery, ErrorResponse, GetSchemasQuery, SchemaResponse,
        UpdateSchemaRequest,
    }, middleware::RequestId, repositories::schema_repository::SchemaQueryParams
};

/// ## GET /schemas
/// Get all schemas with optional filtering by name and/or version.
///
/// Query parameters:
/// - name: Filter schemas by exact name match
/// - version: Filter schemas by exact version match
/// - Both can be combined for precise filtering
///
/// All filtering is performed at the database level for optimal performance.
///
/// Examples:
/// - /schemas - Get all schemas
/// - /schemas?name=web-server-logs - Get all versions of "web-server-logs"
/// - /schemas?version=1.0.0 - Get all schemas with version "1.0.0"
/// - /schemas?name=web-server-logs&version=1.0.0 - Get specific schema by name+version
pub async fn get_schemas(
    State(state): State<AppState>,
    Query(query): Query<GetSchemasQuery>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    let repo_params = SchemaQueryParams::from(query);

    match state
        .schema_service
        .get_all_schemas(Some(repo_params))
        .await
    {
        Ok(schemas) => {
            let schema_responses: Vec<SchemaResponse> = schemas
                .into_iter()
                .map(|schema| SchemaResponse::from(schema))
                .collect();

            Ok(Json(json!({ "schemas": schema_responses })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::with_request_id("INTERNAL_ERROR", e.to_string(), &request_id)),
        )),
    }
}

/// ## GET /schemas/{schema_name}/{schema_version}
/// Get one schema with matching name and version.
pub async fn get_schema_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema name or version cannot be empty",
                &request_id,
            )),
        ));
    }

    match state
        .schema_service
        .get_by_name_and_version(&schema_name, &schema_version)
        .await
    {
        Ok(Some(schema)) => Ok(Json(SchemaResponse::from(schema))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::with_request_id(
                "NOT_FOUND",
                format!(
                    "Schema with name '{}' and version '{}' not found",
                    schema_name, schema_version,
                ),
                &request_id,
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::with_request_id("INTERNAL_ERROR", e.to_string(), &request_id)),
        )),
    }
}

/// ## GET /schemas/{schema_id}
/// Get one schema with matching id.
pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
                &request_id,
            )),
        ));
    }

    match state.schema_service.get_schema_by_id(id).await {
        Ok(Some(schema)) => Ok(Json(SchemaResponse::from(schema))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::with_request_id(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
                &request_id,
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::with_request_id("INTERNAL_ERROR", e.to_string(), &request_id)),
        )),
    }
}

/// ## POST /schemas
/// Create a new schema.
pub async fn create_schema(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateSchemaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema name cannot be empty",
                &request_id,
            )),
        ));
    }

    if payload.version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema version cannot be empty",
                &request_id,
            )),
        ));
    }

    match state
        .schema_service
        .create_schema(
            payload.name,
            payload.version,
            payload.description,
            payload.schema_definition,
        )
        .await
    {
        Ok(schema) => {
            let schema_id = schema.id;
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                format!("/schemas/{}", schema_id).parse().unwrap(),
            );

            Ok((
                StatusCode::CREATED,
                headers,
                Json(SchemaResponse::from(schema)),
            ))
        }
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, error_code) = if error_msg.contains("already exists") {
                (StatusCode::CONFLICT, "SCHEMA_CONFLICT")
            } else if error_msg.contains("Invalid JSON Schema")
                || error_msg.contains("Schema definition must be")
            {
                (StatusCode::BAD_REQUEST, "INVALID_SCHEMA")
            } else {
                (StatusCode::BAD_REQUEST, "CREATION_FAILED")
            };

            Err((status_code, Json(ErrorResponse::with_request_id(error_code, error_msg, &request_id))))
        }
    }
}

/// ## PUT /schemas/{schema_id}
/// Update an existing schema.
pub async fn update_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<UpdateSchemaRequest>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
                &request_id
            )),
        ));
    }

    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema name cannot be empty",
                &request_id,
            )),
        ));
    }

    match state
        .schema_service
        .update_schema(
            id,
            payload.name,
            payload.version,
            payload.description,
            payload.schema_definition,
        )
        .await
    {
        Ok(Some(schema)) => Ok(Json(SchemaResponse::from(schema))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::with_request_id(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
                &request_id,
            )),
        )),
        Err(e) => {
            let error_msg = e.to_string();
            let (status_code, error_code) = if error_msg.contains("already exists") {
                (StatusCode::CONFLICT, "SCHEMA_CONFLICT")
            } else if error_msg.contains("Invalid JSON Schema")
                || error_msg.contains("Schema definition must be")
            {
                (StatusCode::BAD_REQUEST, "INVALID_SCHEMA")
            } else {
                (StatusCode::BAD_REQUEST, "UPDATE_FAILED")
            };

            Err((status_code, Json(ErrorResponse::with_request_id(error_code, error_msg, &request_id))))
        }
    }
}

/// ## DELETE /schema/{schema_id}
/// Delete a schema.
pub async fn delete_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteSchemaQuery>,
    Extension(request_id): Extension<RequestId>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::with_request_id(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
                &request_id,
            )),
        ));
    }

    let force = params.force.unwrap_or(false);

    match state.schema_service.delete_schema(id, force).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::with_request_id(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
                &request_id,
            )),
        )),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Cannot delete schema")
                && error_msg.contains("log(s) are associated")
            {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse::with_request_id("SCHEMA_HAS_LOGS", error_msg, &request_id)),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::with_request_id("DELETION_FAILED", error_msg, &request_id)),
                ))
            }
        }
    }
}
