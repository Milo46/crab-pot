use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct LogQueryParams {
    pub date_begin: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub json_filters: Option<Value>,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaQueryParams {
    pub name: Option<String>,
    pub version: Option<String>,
}
