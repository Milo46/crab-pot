use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    SchemasRead,
    SchemasWrite,
    SchemasDelete,
    LogsRead,
    LogsWrite,
    LogsDelete,
    WsConnect,
}
