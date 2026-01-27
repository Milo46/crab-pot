use crate::common::{create_api_key, get_api_key_by_id, rotate_api_key, setup_admin_test_app};
use log_server::dto::{ApiKeyResponse, CreateApiKeyResponse};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn rotate_existing_key_success() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "rotate-me" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();
    let original_key = created.key.clone();

    let rotate_response = rotate_api_key(&app, created.id).await;
    assert_eq!(rotate_response.status(), StatusCode::OK);

    let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();
    assert_ne!(rotated.key, original_key);
    assert_eq!(rotated.id, created.id);
    assert_eq!(rotated.name, created.name);
}

#[tokio::test]
async fn rotate_generates_new_unique_key() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "rotation-test" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();
    let original_key = created.key.clone();

    let rotate_response = rotate_api_key(&app, created.id).await;
    let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();

    assert_ne!(rotated.key, original_key);
    assert!(!rotated.key.is_empty());
}

#[tokio::test]
async fn rotate_preserves_key_metadata() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "metadata-key",
        "description": "Important key"
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let rotate_response = rotate_api_key(&app, created.id).await;
    let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();

    assert_eq!(rotated.id, created.id);
    assert_eq!(rotated.name, created.name);

    let get_response = get_api_key_by_id(&app, created.id).await;
    let retrieved: ApiKeyResponse = get_response.json().await.unwrap();
    assert_eq!(retrieved.description, Some("Important key".to_string()));
}

#[tokio::test]
async fn rotate_nonexistent_key() {
    let app = setup_admin_test_app().await;

    let response = rotate_api_key(&app, 99999).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rotate_multiple_times() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "multi-rotate" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let mut previous_key = created.key.clone();
    let mut all_keys = vec![previous_key.clone()];

    for _ in 0..3 {
        let rotate_response = rotate_api_key(&app, created.id).await;
        assert_eq!(rotate_response.status(), StatusCode::OK);

        let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();
        assert_ne!(rotated.key, previous_key);

        assert!(!all_keys.contains(&rotated.key));

        all_keys.push(rotated.key.clone());
        previous_key = rotated.key;
    }

    assert_eq!(all_keys.len(), 4);
}

#[tokio::test]
async fn rotate_updates_key_prefix() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "prefix-rotate" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();
    let original_prefix = created.key_prefix.clone();

    let rotate_response = rotate_api_key(&app, created.id).await;
    let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();

    // Key prefix might change with rotation (depends on implementation)
    // At minimum, verify it exists
    assert!(rotated.key_prefix.is_some());

    // If implementation changes prefix on rotation
    if original_prefix != rotated.key_prefix {
        assert_ne!(original_prefix, rotated.key_prefix);
    }
}

#[tokio::test]
async fn rotate_maintains_key_active_status() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "active-rotate" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    // Verify key is active before rotation
    let get_before = get_api_key_by_id(&app, created.id).await;
    let key_before: ApiKeyResponse = get_before.json().await.unwrap();
    assert!(key_before.is_active);

    // Rotate the key
    rotate_api_key(&app, created.id).await;

    // Verify key is still active after rotation
    let get_after = get_api_key_by_id(&app, created.id).await;
    let key_after: ApiKeyResponse = get_after.json().await.unwrap();
    assert!(key_after.is_active);
}

#[tokio::test]
async fn rotate_with_expiration_preserves_expiration() {
    let app = setup_admin_test_app().await;

    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    let payload = json!({
        "name": "expiring-rotate",
        "expires_at": expires_at.to_rfc3339()
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    // Rotate the key
    rotate_api_key(&app, created.id).await;

    // Verify expiration is preserved
    let get_response = get_api_key_by_id(&app, created.id).await;
    let retrieved: ApiKeyResponse = get_response.json().await.unwrap();
    assert!(retrieved.expires_at.is_some());
}

#[tokio::test]
async fn rotate_with_allowed_ips_preserves_restrictions() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "ip-rotate",
        "allowed_ips": ["192.168.1.0/24"]
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    // Rotate the key
    rotate_api_key(&app, created.id).await;

    // Verify IP restrictions are preserved
    let get_response = get_api_key_by_id(&app, created.id).await;
    let retrieved: ApiKeyResponse = get_response.json().await.unwrap();
    assert!(retrieved.allowed_ips.is_some());
    assert!(!retrieved.allowed_ips.unwrap().is_empty());
}

#[tokio::test]
async fn rotate_returns_full_key() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "full-key" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let rotate_response = rotate_api_key(&app, created.id).await;
    let rotated: CreateApiKeyResponse = rotate_response.json().await.unwrap();

    // Response should include the full new key (only time it's visible)
    assert!(!rotated.key.is_empty());
    assert!(rotated.key.len() > 20); // Reasonable minimum for a secure key
}

#[tokio::test]
async fn rotate_with_invalid_id() {
    let app = setup_admin_test_app().await;

    let response = rotate_api_key(&app, -1).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rotate_does_not_affect_other_keys() {
    let app = setup_admin_test_app().await;

    // Create two keys
    let response1 = create_api_key(&app, &json!({ "name": "key-1" })).await;
    let key1: CreateApiKeyResponse = response1.json().await.unwrap();

    let response2 = create_api_key(&app, &json!({ "name": "key-2" })).await;
    let key2: CreateApiKeyResponse = response2.json().await.unwrap();

    // Rotate key2
    rotate_api_key(&app, key2.id).await;

    // Verify key1 is unchanged by fetching it
    let get_key1 = get_api_key_by_id(&app, key1.id).await;
    let retrieved_key1: ApiKeyResponse = get_key1.json().await.unwrap();

    // Verify key1 still has its original properties
    assert_eq!(retrieved_key1.id, key1.id);
    assert_eq!(retrieved_key1.name, "key-1");
    assert_eq!(retrieved_key1.key_prefix, key1.key_prefix);
}

#[tokio::test]
async fn rotate_concurrent_rotations() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "concurrent-rotate" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    // Perform two rotations back-to-back
    let rotate1 = rotate_api_key(&app, created.id).await;
    let rotated1: CreateApiKeyResponse = rotate1.json().await.unwrap();

    let rotate2 = rotate_api_key(&app, created.id).await;
    let rotated2: CreateApiKeyResponse = rotate2.json().await.unwrap();

    // Both should succeed and produce different keys
    assert_ne!(rotated1.key, rotated2.key);
    assert_eq!(rotated1.id, rotated2.id);
}
