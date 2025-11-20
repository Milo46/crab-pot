use reqwest::StatusCode;

use crate::common::{TestContext, valid_schema_payload};

#[tokio::test]
async fn lists_all_schemas() {
    let ctx = TestContext::new().await;

    let initial_response = ctx.client
        .get(&format!("{}/schemas", ctx.base_url))
        .send()
        .await
        .unwrap();
    let initial_data: serde_json::Value = initial_response.json().await.unwrap();
    let initial_count = initial_data["schemas"].as_array().unwrap().len();

    ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("list-test-1"))
        .send()
        .await
        .unwrap();

    ctx.client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("list-test-2"))
        .send()
        .await
        .unwrap();
    
    let response = ctx.client
        .get(&format!("{}/schemas", ctx.base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let data: serde_json::Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();
    assert_eq!(schemas.len(), initial_count + 2, 
        "Expected {} schemas (initial {} + 2 new), but got {}", 
        initial_count + 2, initial_count, schemas.len());
    
    let schema_names: Vec<&str> = schemas.iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(schema_names.contains(&"list-test-1"));
    assert!(schema_names.contains(&"list-test-2"));
}
