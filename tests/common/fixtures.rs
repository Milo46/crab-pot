use serde::{Deserialize, Serialize};
use serde_json::json;

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
                "message": {
                    "type": "string"
                }
            },
            "required": [ "message" ]
        }
    })
}

pub fn valid_log_payload(schema_id: &str) -> serde_json::Value {
    json!({
        "schema_id": schema_id,
        "log_data": {
            "message": "Test log message"
        }
    })
}

pub fn valid_log_payload_with_message(schema_id: &str, message: &str) -> serde_json::Value {
    json!({
        "schema_id": schema_id,
        "log_data": {
            "message": message,
        }
    })
}
