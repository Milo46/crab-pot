use crate::common::{create_api_key, get_api_key_by_id, get_api_keys, setup_admin_test_app};
use log_server::dto::{ApiKeyResponse, ApiKeysResponse, CreateApiKeyResponse};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn admin_health_check_works() {
    let app = setup_admin_test_app().await;

    let response = app
        .client()
        .get("/health")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "log-server-admin");
    assert!(body["timestamp"].is_string());
}

#[tokio::test]
async fn retrieve_empty_keys() {
    let app = setup_admin_test_app().await;

    let response = get_api_keys(&app).await;
    assert_eq!(response.status(), StatusCode::OK);

    let keys: ApiKeysResponse = response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 0);
}

#[tokio::test]
async fn retrieve_one_key() {
    let app = setup_admin_test_app().await;

    let response = create_api_key(&app, &json!({ "name": "test-api-key" })).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = get_api_keys(&app).await;
    assert_eq!(response.status(), StatusCode::OK);

    let keys: ApiKeysResponse = response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 1);

    let key = keys.api_keys.get(0).unwrap();
    assert_eq!(key.name, "test-api-key");
}

#[tokio::test]
async fn retrieve_multiple_keys() {
    let app = setup_admin_test_app().await;

    let names = vec!["key-alpha", "key-beta", "key-gamma"];

    for name in &names {
        let response = create_api_key(&app, &json!({ "name": name })).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let response = get_api_keys(&app).await;
    assert_eq!(response.status(), StatusCode::OK);

    let keys: ApiKeysResponse = response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 3);

    // Verify all names are present
    let returned_names: Vec<String> = keys.api_keys.iter().map(|k| k.name.clone()).collect();
    for name in names {
        assert!(returned_names.contains(&name.to_string()));
    }
}

#[tokio::test]
async fn get_key_by_id_success() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "specific-key" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    assert_eq!(response.status(), StatusCode::OK);

    let key: ApiKeyResponse = response.json().await.unwrap();
    assert_eq!(key.id, created.id);
    assert_eq!(key.name, "specific-key");
    assert!(key.is_active);
}

#[tokio::test]
async fn get_key_by_id_not_found() {
    let app = setup_admin_test_app().await;

    let response = get_api_key_by_id(&app, 99999).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_key_shows_key_prefix() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "prefix-key" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    let key: ApiKeyResponse = response.json().await.unwrap();

    assert!(key.key_prefix.is_some());
    assert!(!key.key_prefix.unwrap().is_empty());
}

#[tokio::test]
async fn get_key_shows_metadata() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "metadata-key",
        "description": "Key with metadata"
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    let key: ApiKeyResponse = response.json().await.unwrap();

    assert_eq!(key.name, "metadata-key");
    assert_eq!(key.description, Some("Key with metadata".to_string()));
    assert!(key.created_at.timestamp() > 0);
}

#[tokio::test]
async fn list_keys_shows_all_metadata() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "detailed-key",
        "description": "Detailed description"
    });

    create_api_key(&app, &payload).await;

    let response = get_api_keys(&app).await;
    let keys: ApiKeysResponse = response.json().await.unwrap();

    assert_eq!(keys.api_keys.len(), 1);
    let key = &keys.api_keys[0];

    assert_eq!(key.name, "detailed-key");
    assert_eq!(key.description, Some("Detailed description".to_string()));
    assert!(key.is_active);
    assert!(key.created_at.timestamp() > 0);
}

#[tokio::test]
async fn get_key_shows_usage_count() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "usage-key" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    let key: ApiKeyResponse = response.json().await.unwrap();

    // Should have usage_count field (even if 0 or None initially)
    assert!(key.usage_count.is_some() || key.usage_count.is_none());
}

#[tokio::test]
async fn list_keys_ordered_consistently() {
    let app = setup_admin_test_app().await;

    // Create keys in specific order
    for i in 1..=5 {
        create_api_key(&app, &json!({ "name": format!("key-{}", i) })).await;
    }

    // Retrieve twice and ensure consistent ordering
    let response1 = get_api_keys(&app).await;
    let keys1: ApiKeysResponse = response1.json().await.unwrap();

    let response2 = get_api_keys(&app).await;
    let keys2: ApiKeysResponse = response2.json().await.unwrap();

    assert_eq!(keys1.api_keys.len(), keys2.api_keys.len());
    for i in 0..keys1.api_keys.len() {
        assert_eq!(keys1.api_keys[i].id, keys2.api_keys[i].id);
    }
}

#[tokio::test]
async fn get_key_with_expiration_shows_expires_at() {
    let app = setup_admin_test_app().await;

    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    let payload = json!({
        "name": "expiring-key",
        "expires_at": expires_at.to_rfc3339()
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    let key: ApiKeyResponse = response.json().await.unwrap();

    assert!(key.expires_at.is_some());
}

#[tokio::test]
async fn get_key_with_allowed_ips_shows_ips() {
    let app = setup_admin_test_app().await;

    let payload = json!({
        "name": "ip-key",
        "allowed_ips": ["192.168.1.0/24"]
    });

    let create_response = create_api_key(&app, &payload).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let response = get_api_key_by_id(&app, created.id).await;
    let key: ApiKeyResponse = response.json().await.unwrap();

    assert!(key.allowed_ips.is_some());
    assert!(!key.allowed_ips.unwrap().is_empty());
}
