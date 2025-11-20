use serde_json::json;

pub const TEST_SCHEMA_NAME: &str = "test-schema";
pub const TEST_SCHEMA_VERSION: &str = "1.0.0";

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
