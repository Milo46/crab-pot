use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct QueryParams {
    pub page: i32,
    pub limit: i32,
    pub date_begin: Option<DateTime<Utc>>,
    pub date_end: Option<DateTime<Utc>>,
    pub filters: Option<Value>,
}
