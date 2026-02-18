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
        schema_dto::CursorSchemasResponse, CreateSchemaRequest, DeleteSchemaQuery, GetSchemasQuery,
        SchemaResponse, UpdateSchemaRequest,
    },
    error::WithRequestId,
    middleware::RequestId,
    models::SchemaQueryParams,
    AppResult, AppState,
};

pub async fn get_schemas(
    State(state): State<AppState>,
    Query(query): Query<GetSchemasQuery>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<CursorSchemasResponse>> {
    let filters = SchemaQueryParams {
        name: query.name,
        version: query.version,
    };

    let (schemas, cursor_metadata) = state
        .schema_service
        .get_cursor_schemas(query.cursor, query.limit, filters)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(CursorSchemasResponse::new(schemas, cursor_metadata)))
}

pub async fn get_schema_by_name_latest(
    State(state): State<AppState>,
    Path(schema_name): Path<String>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<SchemaResponse>> {
    let schema = state
        .schema_service
        .get_schema_by_name(&schema_name)
        .await
        .with_req_id(&request_id)?;

    Ok(Json(SchemaResponse::from(schema)))
}

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

pub async fn get_schemas_initial_cursor(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> AppResult<Json<serde_json::Value>> {
    let cursor = state
        .schema_service
        .get_initial_cursor()
        .await
        .with_req_id(&request_id)?;

    Ok(Json(serde_json::json!({
        "initial_cursor": cursor
    })))
}
