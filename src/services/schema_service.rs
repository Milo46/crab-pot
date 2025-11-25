use crate::error::{AppError, AppResult};
use crate::models::Schema;
use crate::repositories::log_repository::{LogRepository, LogRepositoryTrait};
use crate::repositories::schema_repository::{
    SchemaQueryParams, SchemaRepository, SchemaRepositoryTrait,
};
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SchemaService {
    repository: Arc<SchemaRepository>,
    log_repository: Arc<LogRepository>,
}

impl SchemaService {
    pub fn new(repository: Arc<SchemaRepository>, log_repository: Arc<LogRepository>) -> Self {
        Self {
            repository,
            log_repository,
        }
    }

    pub async fn get_all_schemas(
        &self,
        params: Option<SchemaQueryParams>,
    ) -> AppResult<Vec<Schema>> {
        self.repository.get_all(params).await
    }

    pub async fn get_schema_by_id(&self, id: Uuid) -> AppResult<Option<Schema>> {
        self.repository.get_by_id(id).await
    }

    pub async fn get_by_name_and_version(
        &self,
        name: &str,
        version: &str,
    ) -> AppResult<Option<Schema>> {
        self.repository.get_by_name_and_version(name, version).await
    }

    pub async fn create_schema(
        &self,
        name: String,
        version: String,
        description: Option<String>,
        schema_definition: Value,
    ) -> AppResult<Schema> {
        self.validate_schema_definition(&schema_definition)?;

        let existing = self
            .repository
            .get_by_name_and_version(&name, &version)
            .await?;
        if existing.is_some() {
            return Err(AppError::Conflict(format!(
                "Schema with name '{}' and version '{}' already exists",
                name, version
            )));
        }

        let now = Utc::now();
        let schema = Schema {
            id: Uuid::new_v4(),
            name,
            version,
            description,
            schema_definition,
            created_at: now,
            updated_at: now,
        };

        self.repository.create(&schema).await
    }

    pub async fn update_schema(
        &self,
        id: Uuid,
        name: String,
        version: String,
        description: Option<String>,
        schema_definition: Value,
    ) -> AppResult<Option<Schema>> {
        self.validate_schema_definition(&schema_definition)?;

        let existing_schema = self.repository.get_by_id(id).await?;
        if existing_schema.is_none() {
            return Ok(None);
        }

        let new_schema = self
            .repository
            .get_by_name_and_version(&name, &version)
            .await?;
        if let Some(existing) = new_schema {
            if existing.id != id {
                return Err(AppError::Conflict(format!(
                    "Schema with name '{}' and version '{}' already exists with a different ID",
                    name, version
                )));
            }
        }

        let updated_schema = Schema {
            id,
            name,
            version,
            description,
            schema_definition,
            created_at: existing_schema.unwrap().created_at, // keep original creation time
            updated_at: Utc::now(),
        };

        self.repository.update(id, &updated_schema).await
    }

    pub async fn delete_schema(&self, id: Uuid, force: bool) -> AppResult<bool> {
        let schema = self.repository.get_by_id(id).await?;
        if schema.is_none() {
            return Ok(false);
        }

        let log_count = self.log_repository.count_by_schema_id(id).await?;

        if log_count > 0 && !force {
            return Err(AppError::Conflict(format!(
                "Cannot delete schema: {} log(s) are associated with this schema. Use force=true to delete schema and all associated logs.",
                log_count
            )));
        }

        if force && log_count > 0 {
            let deleted_logs = self.log_repository.delete_by_schema_id(id).await?;
            tracing::info!("Deleted {} logs for schema {}", deleted_logs, id);
        }

        self.repository.delete(id).await
    }

    // Business logic: validate schema definition against JSON Schema meta-schema
    fn validate_schema_definition(&self, schema_definition: &Value) -> AppResult<()> {
        if !schema_definition.is_object() {
            return Err(AppError::ValidationError(
                "Schema definition must be a JSON object".to_string(),
            ));
        }

        let _compiled = jsonschema::validator_for(schema_definition)
            .map_err(|e| AppError::SchemaValidationError(format!("Invalid JSON Schema: {}", e)))?;

        Ok(())

        /*
        use serde_json::json;

        // JSON Schema Draft 7 meta-schema (simplified - in production you'd load the full one)
        let meta_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "type": {"type": "string"},
                "properties": {"type": "object"},
                "required": {"type": "array"},
                "additionalProperties": {"type": "boolean"}
            }
        });

        let meta_validator = jsonschema::JSONSchema::compile(&meta_schema)
            .map_err(|e| anyhow!("Failed to compile meta-schema: {}", e))?;

        if let Err(errors) = meta_validator.validate(schema_definition) {
            let error_messages: Vec<String> = errors
                .map(|error| format!("Meta-schema validation error: {}", error))
                .collect();
            return Err(anyhow!("Schema does not conform to JSON Schema Draft 7: {}",
                             error_messages.join("; ")));
        }
        */
    }
}
