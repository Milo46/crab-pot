use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tower_http::trace::MakeSpan;
use tracing::Span;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }

    pub async fn middleware(mut request: Request, next: Next) -> Response {
        let request_id = request
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| RequestId(s.to_string()))
            .unwrap_or_else(RequestId::new);

        request.extensions_mut().insert(request_id.clone());

        let mut response = next.run(request).await;

        if let Ok(header_value) = HeaderValue::from_str(&request_id.0) {
            response
                .headers_mut()
                .insert(REQUEST_ID_HEADER, header_value);
        }

        response
    }
}

impl Default for RequestIdLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct RequestIdMakeSpan;

impl<B> MakeSpan<B> for RequestIdMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> Span {
        let request_id = request
            .extensions()
            .get::<RequestId>()
            .map(|r| r.as_str())
            .unwrap_or("unknown");

        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = %request_id,
        )
    }
}
