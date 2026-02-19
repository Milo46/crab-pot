use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header::{HeaderName, HeaderValue}},
    middleware::Next,
    response::{Response, IntoResponse},
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
    let rate_limit_burst = api_key.rate_limit_burst.unwrap_or(20) as u32;
    
    if let Err(err) = app_state.rate_limiter.check_rate_limit(
        &key_hash,
        rate_limit_per_second,
        rate_limit_burst,
    ) {
        tracing::warn!(
            "Rate limit exceeded for key: {} - {}",
            api_key.display_key(),
            err
        );
        
        // Create 429 response with rate limit headers
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
    
    // Get rate limit status for response headers
    let rate_limit_status = app_state.rate_limiter.get_status(
        &key_hash,
        rate_limit_per_second,
        rate_limit_burst,
    );

    tokio::spawn(async move {
        let _ = app_state.api_key_service.update_usage(&key_hash).await;
    });

    request.extensions_mut().insert(Arc::new(api_key));
    
    let mut response = next.run(request).await;
    
    // Add rate limit headers to all responses
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

// pub async fn api_key_middleware_debug(
//     State(pool): State<PgPool>,
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     mut request: Request,
//     next: Next,
// ) -> Result<Response, StatusCode> {
//     println!("=== API Key Middleware Debug ===");

//     // Step 1: Check Authorization header
//     let auth_header = match request.headers().get("Authorization") {
//         Some(header) => {
//             println!("✓ Authorization header found");
//             match header.to_str() {
//                 Ok(h) => {
//                     println!("✓ Header value: {}", h);
//                     h
//                 }
//                 Err(e) => {
//                     println!("✗ Failed to parse header as string: {:?}", e);
//                     return Err(StatusCode::UNAUTHORIZED);
//                 }
//             }
//         }
//         None => {
//             println!("✗ No Authorization header found");
//             return Err(StatusCode::UNAUTHORIZED);
//         }
//     };

//     // Step 2: Extract Bearer token
//     let plain_key = match auth_header.strip_prefix("Bearer ") {
//         Some(key) => {
//             println!("✓ Bearer token extracted");
//             println!("  Plain key: {}", key);
//             key
//         }
//         None => {
//             println!("✗ Authorization header doesn't start with 'Bearer '");
//             println!("  Header value: {}", auth_header);
//             return Err(StatusCode::UNAUTHORIZED);
//         }
//     };

//     // Step 3: Hash the key
//     let key_hash = ApiKeyRepository::hash_key(plain_key);
//     println!("✓ Key hashed");
//     println!("  Hash: {}", key_hash);

//     // Step 4: Query database
//     println!("→ Querying database...");
//     let repo = ApiKeyRepository::new(pool);

//     match repo.find_valid_by_hash(&key_hash).await {
//         Ok(Some(api_key)) => {
//             println!("✓ Key found in database");
//             println!("  Key ID: {}", api_key.id);
//             println!("  Key name: {}", api_key.name);
//             println!("  Is active: {}", api_key.is_active);
//             println!("  Expires at: {:?}", api_key.expires_at);

//             // Store in request extensions
//             request.extensions_mut().insert(Arc::new(api_key));

//             // Update usage asynchronously
//             tokio::spawn(async move {
//                 let _ = repo.update_usage(&key_hash).await;
//             });

//             println!("✓ Authentication successful");
//             Ok(next.run(request).await)
//         }
//         Ok(None) => {
//             println!("✗ Key not found in database or invalid");
//             println!("  Searched for hash: {}", key_hash);
//             Err(StatusCode::UNAUTHORIZED)
//         }
//         Err(e) => {
//             println!("✗ Database error: {:?}", e);
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }
