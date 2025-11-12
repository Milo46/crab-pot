use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Schema {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_definition: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
