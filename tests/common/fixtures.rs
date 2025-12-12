use serde_json::json;

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

