use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{repositories::schema_repository::SchemaQueryParams, Schema};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Schema> for SchemaResponse {
    fn from(schema: Schema) -> Self {
        SchemaResponse {
            id: schema.id,
            name: schema.name,
            version: schema.version,
            description: schema.description,
            schema_definition: schema.schema_definition,
            created_at: schema.created_at.to_rfc3339(),
            updated_at: schema.updated_at.to_rfc3339(),
        }
    }
}

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
pub struct DeleteSchemaQuery {
    pub force: Option<bool>,
}
