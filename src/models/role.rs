use crate::models::permission::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "api_key_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Reader,
    Writer,
    SchemaManager,
    LogManager,
    Admin,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            Role::Reader => &[SchemasRead, LogsRead, WsConnect],
            Role::Writer => &[SchemasRead, LogsRead, LogsWrite, WsConnect],
            Role::SchemaManager => &[SchemasRead, SchemasWrite, SchemasDelete, LogsRead],
            Role::LogManager => &[SchemasRead, LogsRead, LogsWrite, LogsDelete, WsConnect],
            Role::Admin => &[
                SchemasRead,
                SchemasWrite,
                SchemasDelete,
                LogsRead,
                LogsWrite,
                LogsDelete,
                WsConnect,
            ],
        }
    }

    pub fn has_permission(self, p: Permission) -> bool {
        self.permissions().contains(&p)
    }
}
