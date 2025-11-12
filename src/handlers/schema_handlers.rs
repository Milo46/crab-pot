use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AppState;

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
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
    pub created_at: String,
    pub updated_at: String,
}

use super::ErrorResponse;

// Handler functions - only responsible for HTTP concerns
pub async fn get_schemas(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    match state.schema_service.get_all_schemas().await {
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
            Json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

pub async fn get_schema_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema ID cannot be empty".to_string(),
            }),
        ));
    }

    match state.schema_service.get_schema_by_id(&id).await {
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
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Schema with id '{}' not found", id),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

pub async fn create_schema(
    State(state): State<AppState>,
    Json(payload): Json<CreateSchemaRequest>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema name cannot be empty".to_string(),
            }),
        ));
    }

    if payload.version.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema version cannot be empty".to_string(),
            }),
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
        Ok(schema) => Ok(Json(SchemaResponse {
            id: schema.id,
            name: schema.name,
            version: schema.version,
            description: schema.description,
            schema_definition: schema.schema_definition,
            created_at: schema.created_at.to_rfc3339(),
            updated_at: schema.updated_at.to_rfc3339(),
        })),
        Err(e) => {
            let status_code = if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };
            
            Err((
                status_code,
                Json(ErrorResponse {
                    error: "CREATION_FAILED".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

pub async fn update_schema(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSchemaRequest>,
) -> Result<Json<SchemaResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema ID cannot be empty".to_string(),
            }),
        ));
    }

    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema name cannot be empty".to_string(),
            }),
        ));
    }

    match state.schema_service
        .update_schema(
            &id,
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
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Schema with id '{}' not found", id),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "UPDATE_FAILED".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

pub async fn delete_schema(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Input validation
    if id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_INPUT".to_string(),
                message: "Schema ID cannot be empty".to_string(),
            }),
        ));
    }

    match state.schema_service.delete_schema(&id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: format!("Schema with id '{}' not found", id),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "DELETION_FAILED".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
