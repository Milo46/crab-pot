use axum::{
    body::Body,
    extract::Request,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde_json::json;

const VALID_API_KEY: &str = "secret-key";

pub async fn api_key_middleware(req: Request, next: Next) -> Response<Body> {
    let api_key = req.headers().get("X-API-KEY").and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if key == VALID_API_KEY => next.run(req).await,
        _ => {
            let error_response = Json(json!({
                "error": "UNAUTHORIZED",
                "message": "Invalid or missing X-API-KEY header"
            }));

            (StatusCode::UNAUTHORIZED, error_response).into_response()
        }
    }
}
