use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{services::api_key_service::ApiKeyService, AppState};

pub async fn api_key_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let plain_key = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let key_hash = ApiKeyService::hash_key(plain_key);

    let api_key = app_state
        .api_key_service
        .find_valid_by_hash(&key_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_ip = addr.ip();
    if !api_key.is_ip_allowed(&client_ip) {
        return Err(StatusCode::FORBIDDEN);
    }

    let rate_limit_per_second = api_key.rate_limit_per_second.unwrap_or(10) as u32;
    let rate_limit_burst = api_key
        .rate_limit_burst
        .map(|b| b as u32)
        .unwrap_or(rate_limit_per_second * 2);

    if let Err(err) =
        app_state
            .rate_limiter
            .check_rate_limit(&key_hash, rate_limit_per_second, rate_limit_burst)
    {
        tracing::warn!(
            "Rate limit exceeded for key: {} - {}",
            api_key.display_key(),
            err
        );

        let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
        let headers = response.headers_mut();
        headers.insert(
            HeaderName::from_static("x-ratelimit-limit"),
            HeaderValue::from(err.limit),
        );
        headers.insert(
            HeaderName::from_static("x-ratelimit-remaining"),
            HeaderValue::from(err.remaining),
        );
        headers.insert(
            HeaderName::from_static("retry-after"),
            HeaderValue::from(err.retry_after),
        );
        return Ok(response);
    }

    let rate_limit_status =
        app_state
            .rate_limiter
            .get_status(&key_hash, rate_limit_per_second, rate_limit_burst);

    tokio::spawn(async move {
        let _ = app_state.api_key_service.update_usage(&key_hash).await;
    });

    request.extensions_mut().insert(Arc::new(api_key));

    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from(rate_limit_status.limit),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from(rate_limit_status.remaining),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from(rate_limit_status.reset_in_secs),
    );

    Ok(response)
}
