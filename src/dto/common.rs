use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::middleware::RequestId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: None,
            request_id: None,
        }
    }

    pub fn with_request_id(
        error: impl Into<String>,
        message: impl Into<String>,
        request_id: &RequestId,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            field_errors: None,
            request_id: Some(request_id.to_string()),
        }
    }

    pub fn set_request_id(mut self, request_id: &RequestId) -> Self {
        self.request_id = Some(request_id.to_string());
        self
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
            request_id: None,
        }
    }
}
