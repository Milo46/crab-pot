use crate::error::{AppError, AppResult};
use crate::models::{Schema, SchemaNameVersion};
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

    pub async fn resolve_schema(&self, schema_ref: &SchemaNameVersion) -> AppResult<Schema> {
        let schema = match &schema_ref.version {
            Some(version) => self
                .repository
                .get_by_name_and_version(&schema_ref.name, version)
                .await
                .map_err(|e| {
                    e.context(format!(
                        "Failed to fetch schema {}:{}",
                        schema_ref.name, version
                    ))
                })?,
            None => self
                .repository
                .get_by_name_latest(&schema_ref.name)
                .await
                .map_err(|e| {
                    e.context(format!("Failed to fetch latest schema {}", schema_ref.name))
                })?,
        };

        schema.ok_or_else(|| {
            let version_str = schema_ref.version.as_deref().unwrap_or("latest");
            AppError::not_found(format!(
                "Schema {}:{} not found",
                schema_ref.name, version_str
            ))
        })
    }

    pub async fn get_schema_id(&self, schema_ref: &SchemaNameVersion) -> AppResult<uuid::Uuid> {
        let schema = self.resolve_schema(schema_ref).await?;
        Ok(schema.id)
    }

    pub async fn validate_log_data(&self, schema_id: Uuid, log_data: &Value) -> AppResult<()> {
        let schema = self.get_schema_by_id(schema_id).await.map_err(|e| {
            e.context(format!(
                "Failed to fetch schema {} for validation",
                schema_id
            ))
        })?;

        let validator = jsonschema::ValidationOptions::default()
            .with_draft(jsonschema::Draft::Draft7)
            .build(&schema.schema_definition)
            .map_err(|e| AppError::internal_error(format!("Invalid JSON schema: {}", e)))?;

        let errors: Vec<_> = validator
            .iter_errors(log_data)
            .map(|e| format!("Validation error at '{}': {}", e.instance_path, e))
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::schema_validation_error(format!(
                "Schema validation failed: {}",
                errors.join("; ")
            )))
        }
    }

    pub async fn get_all_schemas(
        &self,
        params: Option<SchemaQueryParams>,
    ) -> AppResult<Vec<Schema>> {
        self.repository
            .get_all(params)
            .await
            .map_err(|e| e.context("Failed to fetch schemas"))
    }

    pub async fn get_schema_by_id(&self, id: Uuid) -> AppResult<Schema> {
        self.repository
            .get_by_id(id)
            .await
            .map_err(|e| e.context(format!("Failed to fetch schema {}", id)))?
            .ok_or_else(|| AppError::not_found(format!("Schema with id '{}' not found", id)))
    }

    pub async fn get_schema_by_name(&self, name: &str) -> AppResult<Schema> {
        self.repository
            .get_by_name_latest(name)
            .await
            .map_err(|e| e.context(format!("Failed to fetch latest schema '{}'", name)))?
            .ok_or_else(|| AppError::not_found(format!("Schema '{}' not found", name)))
    }

    pub async fn get_by_name_and_version(&self, name: &str, version: &str) -> AppResult<Schema> {
        self.repository
            .get_by_name_and_version(name, version)
            .await
            .map_err(|e| e.context(format!("Failed to fetch schema '{}:{}'", name, version)))?
            .ok_or_else(|| AppError::not_found(format!("Schema '{}:{}' not found", name, version)))
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
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to check for existing schema '{}:{}'",
                    name, version
                ))
            })?;

        if existing.is_some() {
            return Err(AppError::conflict(format!(
                "Schema '{}:{}' already exists",
                name, version
            )));
        }

        let now = Utc::now();
        let schema = Schema {
            id: Uuid::new_v4(),
            name: name.clone(),
            version: version.clone(),
            description,
            schema_definition,
            created_at: now,
            updated_at: now,
        };

        self.repository
            .create(&schema)
            .await
            .map_err(|e| e.context(format!("Failed to create schema '{}:{}'", name, version)))
    }

    pub async fn update_schema(
        &self,
        id: Uuid,
        name: String,
        version: String,
        description: Option<String>,
        schema_definition: Value,
    ) -> AppResult<Schema> {
        if id.is_nil() {
            return Err(AppError::bad_request("Schema ID cannot be empty"));
        }

        self.validate_schema_definition(&schema_definition)?;

        let existing_schema = self
            .get_schema_by_id(id)
            .await
            .map_err(|e| e.context(format!("Failed to fetch schema {}", id)))?;

        let conflicting_schema = self
            .repository
            .get_by_name_and_version(&name, &version)
            .await
            .map_err(|e| {
                e.context(format!(
                    "Failed to check for conflicting schema '{}:{}'",
                    name, version
                ))
            })?;

        if let Some(existing) = conflicting_schema {
            if existing.id != id {
                return Err(AppError::conflict(format!(
                    "Schema '{}:{}' already exists with a different ID",
                    name, version
                )));
            }
        }

        let updated_schema = Schema {
            id,
            name: name.clone(),
            version: version.clone(),
            description,
            schema_definition,
            created_at: existing_schema.created_at, // keep original creation time
            updated_at: Utc::now(),
        };

        self.repository
            .update(id, &updated_schema)
            .await
            .map_err(|e| e.context(format!("Failed to update schema '{}:{}'", name, version)))?
            .ok_or_else(|| AppError::not_found(format!("Schema with id '{}' not found", id)))
    }

    pub async fn delete_schema(&self, id: Uuid, force: bool) -> AppResult<bool> {
        if id.is_nil() {
            return Err(AppError::bad_request("Cannot delete Schema with nil UUID"));
        }

        let schema = self
            .repository
            .get_by_id(id)
            .await
            .map_err(|e| e.context(format!("Failed to fetch schema {}", id)))?;

        if schema.is_none() {
            return Err(AppError::not_found(format!(
                "Schema with id {} not found",
                id
            )));
        }

        let log_count = self
            .log_repository
            .count_by_schema_id(id, None, None, None)
            .await
            .map_err(|e| e.context(format!("Failed to count logs for schema {}", id)))?;

        if log_count > 0 && !force {
            return Err(AppError::conflict(format!(
                "Cannot delete schema: {} log(s) are associated with this schema. Use force=true to delete schema and all associated logs.",
                log_count
            )));
        }

        if force && log_count > 0 {
            let deleted_logs = self
                .log_repository
                .delete_by_schema_id(id)
                .await
                .map_err(|e| e.context(format!("Failed to delete logs for schema {}", id)))?;
            tracing::info!("Deleted {} logs for schema {}", deleted_logs, id);
        }

        self.repository
            .delete(id)
            .await
            .map_err(|e| e.context(format!("Failed to delete schema {}", id)))
    }

    fn validate_schema_definition(&self, schema_definition: &Value) -> AppResult<()> {
        if !schema_definition.is_object() {
            return Err(AppError::validation_error(
                "Schema definition must be a JSON object",
            ));
        }

        jsonschema::validator_for(schema_definition).map_err(|e| {
            AppError::schema_validation_error(format!("Invalid JSON Schema: {}", e))
        })?;

        Ok(())
    }
}
