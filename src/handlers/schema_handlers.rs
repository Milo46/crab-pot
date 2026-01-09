use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::{
        schema_dto::SchemasResponse, CreateSchemaRequest, DeleteSchemaQuery, GetSchemasQuery,
        SchemaResponse, UpdateSchemaRequest,
    },
    error::WithRequestId,
    middleware::RequestId,
    repositories::schema_repository::SchemaQueryParams,
    AppResult, AppState,
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
) -> AppResult<Json<SchemasResponse>> {
    let repo_params = SchemaQueryParams {
        name: query.name,
        version: query.version,
    };

    let schemas = state
        .schema_service
        .get_all_schemas(Some(repo_params))
        .await
        .with_req_id(&request_id)?;

    Ok(Json(SchemasResponse::from(schemas)))
}

/// ## GET /schemas/{schema_name}/{schema_version}
/// Get one schema with matching name and version.
pub async fn get_schema_by_name_and_version(
    State(state): State<AppState>,
    Path((schema_name, schema_version)): Path<(String, String)>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<SchemaResponse>> {
    let schema = state
        .schema_service
        .get_by_name_and_version(&schema_name, &schema_version)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(SchemaResponse::from(schema)))
}

/// ## GET /schemas/{schema_id}
/// Get one schema with matching id.
pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<SchemaResponse>> {
    let schema = state
        .schema_service
        .get_schema_by_id(id)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(SchemaResponse::from(schema)))
}

/// ## POST /schemas
/// Create a new schema.
pub async fn create_schema(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateSchemaRequest>,
) -> AppResult<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| crate::AppError::validation_error(format!("Validation failed: {}", e)))?;

    let schema = state
        .schema_service
        .create_schema(
            payload.name,
            payload.version,
            payload.description,
            payload.schema_definition,
        )
        .await
        .with_req_id(&request_id)?;

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
    payload
        .validate()
        .map_err(|e| crate::AppError::validation_error(format!("Validation failed: {}", e)))?;

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
        .with_req_id(&request_id)?;

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
    let force = params.force.unwrap_or(false);

    state
        .schema_service
        .delete_schema(id, force)
        .await
        .with_req_id(&request_id)?;

    Ok(StatusCode::NO_CONTENT)
}
