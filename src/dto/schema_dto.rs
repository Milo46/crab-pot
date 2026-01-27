use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::Schema;

fn validate_string_not_empty(string: &String) -> Result<(), validator::ValidationError> {
    if string.trim().is_empty() {
        return Err(validator::ValidationError::new("string_empty"));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSchemaRequest {
    #[validate(custom(
        function = "validate_string_not_empty",
        message = "Schema name cannot be empty"
    ))]
    pub name: String,
    #[validate(custom(
        function = "validate_string_not_empty",
        message = "Schema version cannot be empty"
    ))]
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSchemaRequest {
    #[validate(custom(
        function = "validate_string_not_empty",
        message = "Schema name cannot be empty"
    ))]
    pub name: String,
    #[validate(custom(
        function = "validate_string_not_empty",
        message = "Schema version cannot be empty"
    ))]
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemasResponse {
    pub schemas: Vec<SchemaResponse>,
}

impl From<Vec<Schema>> for SchemasResponse {
    fn from(value: Vec<Schema>) -> Self {
        Self {
            schemas: value.into_iter().map(SchemaResponse::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSchemasQuery {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteSchemaQuery {
    pub force: Option<bool>,
}
