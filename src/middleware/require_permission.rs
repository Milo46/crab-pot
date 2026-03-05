use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Extension,
};

use crate::models::{permission::Permission, ApiKey};

pub async fn require_permission(
    State(required): State<Permission>,
    Extension(api_key): Extension<Arc<ApiKey>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if api_key.role.has_permission(required) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
