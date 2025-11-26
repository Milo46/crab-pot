use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<HashMap<String, Vec<String>>>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: None,
        }
    }

    pub fn with_field_errors(
        error: impl Into<String>,
        message: impl Into<String>,
        field_errors: HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: Some(field_errors),
        }
    }
}
