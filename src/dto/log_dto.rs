use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateLogRequest {
    pub schema_id: Uuid,
    pub log_data: Value,
}

#[derive(Debug, Serialize)]
pub struct LogResponse {
    pub id: i32,
    pub schema_id: Uuid,
    pub log_data: Value,
    pub created_at: String,
}
