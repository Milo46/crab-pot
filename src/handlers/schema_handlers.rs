use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    dto::{
        CreateSchemaRequest, DeleteSchemaQuery, GetSchemasQuery, SchemaResponse,
        UpdateSchemaRequest,
    },
    middleware::RequestId,
    repositories::schema_repository::SchemaQueryParams,
    AppError, AppResult, AppState,
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
) -> AppResult<Json<Value>> {
    let repo_params = SchemaQueryParams::from(query);

    let schemas = state
        .schema_service
        .get_all_schemas(Some(repo_params))
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    let schema_responses: Vec<SchemaResponse> =
        schemas.into_iter().map(SchemaResponse::from).collect();

    Ok(Json(json!({ "schemas": schema_responses })))
}

/// ## GET /schemas/{schema_name}/{schema_version}
/// Get one schema with matching name and version.
pub async fn get_schema_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<SchemaResponse>> {
    if schema_name.trim().is_empty() || schema_version.trim().is_empty() {
        return Err(
            AppError::bad_request("Schema name or version cannot be empty")
                .with_request_id(&request_id),
        );
    }

    let schema = state
        .schema_service
        .get_by_name_and_version(&schema_name, &schema_version)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(SchemaResponse::from(schema)))
}

/// ## GET /schemas/{schema_id}
/// Get one schema with matching id.
pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<SchemaResponse>> {
    if id.is_nil() {
        return Err(AppError::bad_request("Schema ID cannot be empty").with_request_id(&request_id));
    }

    let schema = state
        .schema_service
        .get_schema_by_id(id)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(SchemaResponse::from(schema)))
}

/// ## POST /schemas
/// Create a new schema.
pub async fn create_schema(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateSchemaRequest>,
) -> AppResult<impl IntoResponse> {
    if payload.name.trim().is_empty() {
        return Err(
            AppError::bad_request("Schema name cannot be empty").with_request_id(&request_id)
        );
    }

    if payload.version.trim().is_empty() {
        return Err(
            AppError::bad_request("Schema version cannot be empty").with_request_id(&request_id)
        );
    }

    let schema = state
        .schema_service
        .create_schema(
            payload.name,
            payload.version,
            payload.description,
            payload.schema_definition,
        )
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

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

/// ## PUT /schemas/{schema_id}
/// Update an existing schema.
pub async fn update_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<UpdateSchemaRequest>,
) -> AppResult<Json<SchemaResponse>> {
    if id.is_nil() {
        return Err(AppError::bad_request("Schema ID cannot be empty").with_request_id(&request_id));
    }

    if payload.name.trim().is_empty() {
        return Err(
            AppError::bad_request("Schema name cannot be empty").with_request_id(&request_id)
        );
    }

    let schema = state
        .schema_service
        .update_schema(
            id,
            payload.name,
            payload.version,
            payload.description,
            payload.schema_definition,
        )
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    Ok(Json(SchemaResponse::from(schema)))
}

/// ## DELETE /schema/{schema_id}
/// Delete a schema.
pub async fn delete_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteSchemaQuery>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<StatusCode> {
    if id.is_nil() {
        return Err(AppError::bad_request("Schema ID cannot be empty").with_request_id(&request_id));
    }

    let force = params.force.unwrap_or(false);

    let deleted = state
        .schema_service
        .delete_schema(id, force)
        .await
        .map_err(|e| e.with_request_id(&request_id))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(
            AppError::not_found(format!("Schema with id '{}' not found", id))
                .with_request_id(&request_id),
        )
    }
}
