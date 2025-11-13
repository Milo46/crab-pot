use std::sync::Arc;
use crate::models::Log;
use crate::repositories::log_repository::{LogRepository, LogRepositoryTrait};
use crate::repositories::schema_repository::{SchemaRepository, SchemaRepositoryTrait};
use anyhow::{Result, anyhow};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct LogService {
    log_repository: Arc<LogRepository>,
    schema_repository: Arc<SchemaRepository>,
}

impl LogService {
    pub fn new(log_repository: Arc<LogRepository>, schema_repository: Arc<SchemaRepository>) -> Self {
        Self {
            log_repository,
            schema_repository,
        }
    }

    pub async fn get_logs_by_schema_name_and_id(&self, name: &str, version: &str) -> Result<Vec<Log>> {
        let schema = self.schema_repository.get_by_name_and_id(name, version).await?;
        if schema.is_none() {
            return Err(anyhow!("Schema with name:version '{}:{}' not found", name, version));
        }

        self.log_repository.get_by_schema_id(schema.unwrap().id).await
    }

    pub async fn get_log_by_id(&self, id: i32) -> Result<Option<Log>> {
        self.log_repository.get_by_id(id).await
    }

    pub async fn create_log(&self, schema_id: Uuid, log_data: Value) -> Result<Log> {
        let schema = self.schema_repository.get_by_id(schema_id).await?;
        let schema = match schema {
            Some(s) => s,
            None => return Err(anyhow!("Schema with id '{}' not found", schema_id)),
        };

        self.validate_log_against_schema(&log_data, &schema.schema_definition)?;

        let log = Log {
            id: 0, // This will be set by the database
            schema_id,
            log_data,
            created_at: Utc::now(),
        };

        self.log_repository.create(&log).await
    }

    pub async fn delete_log(&self, id: i32) -> Result<bool> {
        self.log_repository.delete(id).await
    }

    fn validate_log_against_schema(&self, log_data: &Value, schema_definition: &Value) -> Result<()> {
        let validator = jsonschema::ValidationOptions::default()
            .with_draft(jsonschema::Draft::Draft7)
            .build(schema_definition)
            .map_err(|e| anyhow!("Invalid JSON schema: {}", e))?;

        let errors: Vec<_> = validator
            .iter_errors(log_data)
            .map(|e| format!("Validation error at '{}': {}", e.instance_path, e))
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Schema validation failed: {}",
                errors.join("; ")
            ))
        }
    }
}
