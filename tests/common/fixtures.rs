use serde::{Deserialize, Serialize};
use serde_json::json;

pub const TEST_SCHEMA_NAME: &str = "test-schema";
pub const TEST_SCHEMA_VERSION: &str = "1.0.0";

/// ErrorResponse matches the JSON structure returned by AppError::into_response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

pub fn valid_schema_payload(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "version": "1.0.0",
        "schema_definition": {
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": [ "message" ]
        }
    })
}

pub fn valid_log_payload(schema_id: uuid::Uuid) -> serde_json::Value {
    json!({
        "schema_id": schema_id,
        "log_data": {
            "message": "Test log message"
        }
    })
}
