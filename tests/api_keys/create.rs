use crate::common::{
    create_api_key, get_api_key_by_id, setup_admin_test_app, valid_api_key_payload,
};
use chrono::{Duration, Utc};
use crab_pot::dto::CreateApiKeyResponse;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn create_valid_key() {
    let app = setup_admin_test_app().await;

    let response = create_api_key(&app, &valid_api_key_payload("first-api-key")).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: CreateApiKeyResponse = response.json().await.unwrap();
    let api_key: String = response_body.key;

    assert_eq!(response_body.name, "first-api-key");
    assert!(!api_key.trim().is_empty());
    assert!(response_body.key_prefix.is_some());
    assert!(response_body.id > 0);
}

#[tokio::test]
async fn create_key_with_description() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "api-key-with-desc",
        "description": "This is a test API key with description"
    });

    let response = create_api_key(&app, &payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: CreateApiKeyResponse = response.json().await.unwrap();
    assert_eq!(response_body.name, "api-key-with-desc");
    assert!(!response_body.key.is_empty());
}

#[tokio::test]
async fn create_key_with_expiration() {
    let app = setup_admin_test_app().await;

    let expires_at = Utc::now() + Duration::days(30);
    let payload = json!({
        "name": "expiring-key",
        "expires_at": expires_at.to_rfc3339()
    });

    let response = create_api_key(&app, &payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: CreateApiKeyResponse = response.json().await.unwrap();
    assert_eq!(response_body.name, "expiring-key");
    assert!(response_body.expires_at.is_some());
}

#[tokio::test]
async fn create_key_with_allowed_ips() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "ip-restricted-key",
        "allowed_ips": ["192.168.1.0/24", "10.0.0.1/32"]
    });

    let response = create_api_key(&app, &payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: CreateApiKeyResponse = response.json().await.unwrap();
    assert_eq!(response_body.name, "ip-restricted-key");
}

#[tokio::test]
async fn create_multiple_keys() {
    let app = setup_admin_test_app().await;

    let names = vec!["key-one", "key-two", "key-three"];
    let mut created_ids = Vec::new();

    for name in names {
        let response = create_api_key(&app, &valid_api_key_payload(name)).await;
        assert_eq!(response.status(), StatusCode::CREATED);

        let response_body: CreateApiKeyResponse = response.json().await.unwrap();
        assert_eq!(response_body.name, name);
        created_ids.push(response_body.id);
    }

    let unique_ids: std::collections::HashSet<_> = created_ids.iter().collect();
    assert_eq!(unique_ids.len(), created_ids.len());
}

#[tokio::test]
async fn create_key_generates_unique_keys() {
    let app = setup_admin_test_app().await;

    let response1 = create_api_key(&app, &valid_api_key_payload("key-1")).await;
    let key1: CreateApiKeyResponse = response1.json().await.unwrap();

    let response2 = create_api_key(&app, &valid_api_key_payload("key-2")).await;
    let key2: CreateApiKeyResponse = response2.json().await.unwrap();

    assert_ne!(key1.key, key2.key);
    assert_ne!(key1.id, key2.id);
}

#[tokio::test]
async fn create_key_persists_to_database() {
    let app = setup_admin_test_app().await;

    let response = create_api_key(&app, &valid_api_key_payload("persistent-key")).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let created: CreateApiKeyResponse = response.json().await.unwrap();

    let get_response = get_api_key_by_id(&app, created.id).await;
    assert_eq!(get_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_empty_name() {
    let app = setup_admin_test_app().await;

    let response = create_api_key(&app, &valid_api_key_payload("")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_missing_name() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "description": "Key without name"
    });

    let response = create_api_key(&app, &payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_invalid_json() {
    let app = setup_admin_test_app().await;

    let response = app
        .client()
        .post("/api-keys")
        .header("Content-Type", "application/json")
        .body("invalid json content")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_key_with_all_fields() {
    let app = setup_admin_test_app().await;

    let expires_at = Utc::now() + Duration::days(90);
    let payload = json!({
        "name": "complete-key",
        "description": "Key with all optional fields",
        "expires_at": expires_at.to_rfc3339(),
        "allowed_ips": ["192.168.1.100/32"]
    });

    let response = create_api_key(&app, &payload).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response_body: CreateApiKeyResponse = response.json().await.unwrap();
    assert_eq!(response_body.name, "complete-key");
    assert!(response_body.expires_at.is_some());
    assert!(!response_body.key.is_empty());
}
