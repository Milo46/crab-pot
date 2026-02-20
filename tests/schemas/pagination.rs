use crab_pot::Schema;
use reqwest::StatusCode;
use serde_json::{json, Value};

use crate::common::{create_schema, get_schemas_with_cursor, setup_test_app, valid_schema_payload};

#[tokio::test]
async fn forward_pagination_fetches_older_schemas() {
    let app = setup_test_app().await;

    for i in 1..=5 {
        create_schema(
            &app,
            &valid_schema_payload(&format!("schema-forward-{}", i)),
        )
        .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = get_schemas_with_cursor(&app, None, 3, "forward").await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 3);
    assert_eq!(data["cursor"]["limit"], 3);

    let next_cursor = data["cursor"]["next_cursor"].as_str();
    assert!(next_cursor.is_some());

    let prev_cursor = data["cursor"]["prev_cursor"].as_str();
    assert!(prev_cursor.is_some());

    let response2 =
        get_schemas_with_cursor(&app, next_cursor.map(String::from), 3, "forward").await;
    assert_eq!(response2.status(), StatusCode::OK);

    let data2: Value = response2.json().await.unwrap();
    let schemas2 = data2["schemas"].as_array().unwrap();
    assert!(schemas2.len() > 0);
}

#[tokio::test]
async fn backward_pagination_fetches_newer_schemas() {
    let app = setup_test_app().await;

    let mut schema_ids = Vec::new();
    for i in 1..=5 {
        let response = create_schema(
            &app,
            &valid_schema_payload(&format!("schema-backward-{}", i)),
        )
        .await;
        let schema: Schema = response.json().await.unwrap();
        schema_ids.push(schema.id.to_string());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let cursor = &schema_ids[1];
    let response = get_schemas_with_cursor(&app, Some(cursor.clone()), 3, "backward").await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();
    assert!(schemas.len() > 0);

    let first_id = schemas[0]["id"].as_str().unwrap();
    assert_ne!(first_id, cursor);
}

#[tokio::test]
async fn prev_cursor_navigates_to_newer_schemas_in_forward_mode() {
    let app = setup_test_app().await;

    for i in 1..=7 {
        create_schema(&app, &valid_schema_payload(&format!("schema-prev-{}", i))).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response1 = get_schemas_with_cursor(&app, None, 3, "forward").await;
    let data1: Value = response1.json().await.unwrap();
    let next_cursor1 = data1["cursor"]["next_cursor"].as_str().map(String::from);

    let response2 = get_schemas_with_cursor(&app, next_cursor1, 3, "forward").await;
    let data2: Value = response2.json().await.unwrap();
    let prev_cursor2 = data2["cursor"]["prev_cursor"].as_str().map(String::from);

    let response3 = get_schemas_with_cursor(&app, prev_cursor2, 3, "backward").await;
    let data3: Value = response3.json().await.unwrap();
    let schemas3 = data3["schemas"].as_array().unwrap();
    assert!(schemas3.len() > 0);
}

#[tokio::test]
async fn pagination_with_name_filter() {
    let app = setup_test_app().await;

    for i in 1..=5 {
        create_schema(&app, &valid_schema_payload(&format!("filtered-{}", i))).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = app
        .auth()
        .get("/schemas")
        .query(&[
            ("name", "filtered-1"),
            ("limit", "10"),
            ("direction", "forward"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let data: Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();

    for schema in schemas {
        let name = schema["name"].as_str().unwrap();
        assert_eq!(name, "filtered-1");
    }
}

#[tokio::test]
async fn pagination_with_version_filter() {
    let app = setup_test_app().await;

    let base_schema = valid_schema_payload("versioned");

    for version in &["1.0.0", "1.0.1", "2.0.0"] {
        let mut schema = base_schema.clone();
        schema["version"] = json!(version);
        create_schema(&app, &schema).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = app
        .auth()
        .get("/schemas")
        .query(&[
            ("version", "1.0.0"),
            ("limit", "10"),
            ("direction", "forward"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let data: Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();

    for schema in schemas {
        let version = schema["version"].as_str().unwrap();
        assert_eq!(version, "1.0.0");
    }
}

#[tokio::test]
async fn empty_result_with_cursor() {
    let app = setup_test_app().await;

    for i in 1..=2 {
        create_schema(&app, &valid_schema_payload(&format!("schema-empty-{}", i))).await;
    }

    let response = app
        .auth()
        .get("/schemas")
        .query(&[("limit", "100"), ("direction", "forward")])
        .send()
        .await
        .unwrap();

    let data: Value = response.json().await.unwrap();
    let _has_more = data["cursor"]["has_more"].as_bool().unwrap();

    assert!(data["cursor"]["limit"].is_number());
}

#[tokio::test]
async fn default_direction_is_forward() {
    let app = setup_test_app().await;

    let mut schema_ids = Vec::new();
    for i in 1..=3 {
        let response = create_schema(
            &app,
            &valid_schema_payload(&format!("schema-default-{}", i)),
        )
        .await;
        let schema: Schema = response.json().await.unwrap();
        schema_ids.push(schema.id.to_string());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let response = app
        .auth()
        .get("/schemas")
        .query(&[("limit", "2")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let data: Value = response.json().await.unwrap();
    assert_eq!(data["cursor"]["limit"], 2);

    let schemas = data["schemas"].as_array().unwrap();
    assert!(schemas.len() >= 2);
}

#[tokio::test]
async fn cursor_metadata_structure() {
    let app = setup_test_app().await;

    for i in 1..=3 {
        create_schema(
            &app,
            &valid_schema_payload(&format!("schema-metadata-{}", i)),
        )
        .await;
    }

    let response = get_schemas_with_cursor(&app, None, 2, "forward").await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: Value = response.json().await.unwrap();
    let cursor = &data["cursor"];

    assert!(cursor["limit"].is_number());
    assert_eq!(cursor["limit"], 2);
    assert!(cursor["has_more"].is_boolean());

    if let Some(next) = cursor["next_cursor"].as_str() {
        assert!(uuid::Uuid::parse_str(next).is_ok());
    }

    if let Some(prev) = cursor["prev_cursor"].as_str() {
        assert!(uuid::Uuid::parse_str(prev).is_ok());
    }
}
