use serde::Serialize;

use crate::models::{permission::Permission, role::Role};

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub key_prefix: String,
    pub name: String,
    pub role: Role,
    pub permissions: Vec<Permission>,
}
