use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::repositories::schema_repository::SchemaQueryParams;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GetSchemasQuery {
    pub name: Option<String>,
    pub version: Option<String>,
}

impl From<GetSchemasQuery> for SchemaQueryParams {
    fn from(query: GetSchemasQuery) -> Self {
        SchemaQueryParams {
            name: query.name,
            version: query.version,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSchemaRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSchemaRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSchemaQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
    pub created_at: String,
    pub updated_at: String,
}

use super::ErrorResponse;

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
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    let repo_params = query.into();

    match state
        .schema_service
        .get_all_schemas(Some(repo_params))
        .await
    {
        Ok(schemas) => {
            let schema_responses: Vec<SchemaResponse> = schemas
                .into_iter()
                .map(|s| SchemaResponse {
                    id: s.id,
                    name: s.name,
                    version: s.version,
                    description: s.description,
                    schema_definition: s.schema_definition,
                    created_at: s.created_at.to_rfc3339(),
                    updated_at: s.updated_at.to_rfc3339(),
                })
                .collect();

            Ok(Json(json!({ "schemas": schema_responses })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string())),
        )),
    }
}

pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
            )),
        ));
    }

    match state.schema_service.get_schema_by_id(id).await {
        Ok(Some(schema)) => Ok(Json(SchemaResponse {
            id: schema.id,
            name: schema.name,
            version: schema.version,
            description: schema.description,
            schema_definition: schema.schema_definition,
            created_at: schema.created_at.to_rfc3339(),
            updated_at: schema.updated_at.to_rfc3339(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
            )),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("INTERNAL_ERROR", e.to_string())),
        )),
    }
}

pub async fn create_schema(
    State(state): State<AppState>,
    Json(payload): Json<CreateSchemaRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name cannot be empty",
            )),
        ));
    }

    if payload.version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema version cannot be empty",
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
                Json(SchemaResponse {
                    id: schema.id,
                    name: schema.name,
                    version: schema.version,
                    description: schema.description,
                    schema_definition: schema.schema_definition,
                    created_at: schema.created_at.to_rfc3339(),
                    updated_at: schema.updated_at.to_rfc3339(),
                }),
            ))
        }
        Err(e) => {
            let status_code = if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };

            Err((
                status_code,
                Json(ErrorResponse::new("CREATION_FAILED", e.to_string())),
            ))
        }
    }
}

pub async fn update_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSchemaRequest>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
            )),
        ));
    }

    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema name cannot be empty",
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
        Ok(Some(schema)) => Ok(Json(SchemaResponse {
            id: schema.id,
            name: schema.name,
            version: schema.version,
            description: schema.description,
            schema_definition: schema.schema_definition,
            created_at: schema.created_at.to_rfc3339(),
            updated_at: schema.updated_at.to_rfc3339(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
            )),
        )),
        Err(e) => {
            let error_msg = e.to_string();
            let status_code = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };

            Err((
                status_code,
                Json(ErrorResponse::new("UPDATE_FAILED", error_msg)),
            ))
        }
    }
}

pub async fn delete_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteSchemaQuery>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "INVALID_INPUT",
                "Schema ID cannot be empty",
            )),
        ));
    }

    let force = params.force.unwrap_or(false);

    match state.schema_service.delete_schema(id, force).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(
                "NOT_FOUND",
                format!("Schema with id '{}' not found", id),
            )),
        )),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Cannot delete schema")
                && error_msg.contains("log(s) are associated")
            {
                Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse::new("SCHEMA_HAS_LOGS", error_msg)),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("DELETION_FAILED", error_msg)),
                ))
            }
        }
    }
}
