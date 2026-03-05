use std::sync::Arc;

use axum::{Extension, Json};

use crate::{dto::MeResponse, models::ApiKey};

pub async fn me(Extension(api_key): Extension<Arc<ApiKey>>) -> Json<MeResponse> {
    Json(MeResponse {
        key_prefix: api_key.display_key(),
        name: api_key.name.clone(),
        role: api_key.role,
        permissions: api_key.role.permissions().to_vec(),
    })
}
