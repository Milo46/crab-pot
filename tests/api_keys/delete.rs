use crate::common::{
    create_api_key, delete_api_key, get_api_key_by_id, get_api_keys, setup_admin_test_app,
};
use log_server::dto::{ApiKeysResponse, CreateApiKeyResponse};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn delete_existing_key_success() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "delete-me" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let delete_response = delete_api_key(&app, created.id).await;
    assert_eq!(delete_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn delete_removes_key_from_database() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "to-delete" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let get_response = get_api_key_by_id(&app, created.id).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let delete_response = delete_api_key(&app, created.id).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_response_after = get_api_key_by_id(&app, created.id).await;
    assert_eq!(get_response_after.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_key() {
    let app = setup_admin_test_app().await;

    let response = delete_api_key(&app, 99999).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_key_removes_from_list() {
    let app = setup_admin_test_app().await;

    create_api_key(&app, &json!({ "name": "key-1" })).await;
    let create_response = create_api_key(&app, &json!({ "name": "key-2" })).await;
    let key_to_delete: CreateApiKeyResponse = create_response.json().await.unwrap();
    create_api_key(&app, &json!({ "name": "key-3" })).await;

    let list_response = get_api_keys(&app).await;
    let keys: ApiKeysResponse = list_response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 3);

    let delete_response = delete_api_key(&app, key_to_delete.id).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let list_response = get_api_keys(&app).await;
    let keys: ApiKeysResponse = list_response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 2);

    let remaining_names: Vec<String> = keys.api_keys.iter().map(|k| k.name.clone()).collect();
    assert!(!remaining_names.contains(&"key-2".to_string()));
    assert!(remaining_names.contains(&"key-1".to_string()));
    assert!(remaining_names.contains(&"key-3".to_string()));
}

#[tokio::test]
async fn delete_same_key_twice() {
    let app = setup_admin_test_app().await;

    let create_response = create_api_key(&app, &json!({ "name": "once-deleted" })).await;
    let created: CreateApiKeyResponse = create_response.json().await.unwrap();

    let delete_response1 = delete_api_key(&app, created.id).await;
    assert_eq!(delete_response1.status(), StatusCode::OK);

    let delete_response2 = delete_api_key(&app, created.id).await;
    assert_eq!(delete_response2.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_all_keys() {
    let app = setup_admin_test_app().await;

    let mut key_ids = Vec::new();
    for i in 1..=5 {
        let response = create_api_key(&app, &json!({ "name": format!("key-{}", i) })).await;
        let created: CreateApiKeyResponse = response.json().await.unwrap();
        key_ids.push(created.id);
    }

    for id in key_ids {
        let delete_response = delete_api_key(&app, id).await;
        assert_eq!(delete_response.status(), StatusCode::OK);
    }

    let list_response = get_api_keys(&app).await;
    let keys: ApiKeysResponse = list_response.json().await.unwrap();
    assert_eq!(keys.api_keys.len(), 0);
}

#[tokio::test]
async fn delete_with_invalid_id_format() {
    let app = setup_admin_test_app().await;

    let response = app
        .client()
        .delete("/api-keys/not-a-number")
        .send()
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn delete_with_negative_id() {
    let app = setup_admin_test_app().await;

    let response = delete_api_key(&app, -1).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_does_not_affect_other_keys() {
    let app = setup_admin_test_app().await;

    let response1 = create_api_key(&app, &json!({ "name": "keep-1" })).await;
    let key1: CreateApiKeyResponse = response1.json().await.unwrap();

    let response2 = create_api_key(&app, &json!({ "name": "delete-this" })).await;
    let key2: CreateApiKeyResponse = response2.json().await.unwrap();

    let response3 = create_api_key(&app, &json!({ "name": "keep-2" })).await;
    let key3: CreateApiKeyResponse = response3.json().await.unwrap();

    delete_api_key(&app, key2.id).await;

    let get_response1 = get_api_key_by_id(&app, key1.id).await;
    assert_eq!(get_response1.status(), StatusCode::OK);

    let get_response3 = get_api_key_by_id(&app, key3.id).await;
    assert_eq!(get_response3.status(), StatusCode::OK);
}
