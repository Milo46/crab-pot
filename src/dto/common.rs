use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CursorMetadata<T> {
    pub limit: i32,
    pub next_cursor: Option<T>,
    /// Cursor for fetching the previous page (toward newer logs).
    /// Currently not supported - always None.
    pub prev_cursor: Option<T>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct DeletedResponse<T> {
    pub deleted: bool,
    pub data: T,
}
