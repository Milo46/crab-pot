use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap, header},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;
use crate::repositories::SchemaQueryParams;

// Request/Response DTOs for API layer
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
    Query(params): Query<SchemaQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.schema_service.get_all_schemas(Some(params)).await {
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
2}

pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("INVALID_INPUT", "Schema ID cannot be empty")),
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
            Json(ErrorResponse::new("NOT_FOUND", format!("Schema with id '{}' not found", id))),
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
            Json(ErrorResponse::new("INVALID_INPUT", "Schema name cannot be empty")),
        ));
    }

    if payload.version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("INVALID_INPUT", "Schema version cannot be empty")),
        ));
    }

    match state.schema_service
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
        },
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
            Json(ErrorResponse::new("INVALID_INPUT", "Schema ID cannot be empty")),
        ));
    }

    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("INVALID_INPUT", "Schema name cannot be empty")),
        ));
    }

    match state.schema_service
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
            Json(ErrorResponse::new("NOT_FOUND", format!("Schema with id '{}' not found", id))),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("UPDATE_FAILED", e.to_string())),
        )),
    }
}

pub async fn delete_schema(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if id.is_nil() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("INVALID_INPUT", "Schema ID cannot be empty")),
        ));
    }

    match state.schema_service.delete_schema(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("NOT_FOUND", format!("Schema with id '{}' not found", id))),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("DELETION_FAILED", e.to_string())),
        )),
    }
}
