use log_server::Schema;
use reqwest::StatusCode;

use crate::common::{valid_schema_payload, TestContext};

#[tokio::test]
async fn retrieves_existing_schema_by_id() {
    let ctx = TestContext::new().await;
    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("get-test"))
        .send()
        .await
        .unwrap();

    let schema: Schema = schema_response.json().await.unwrap();

    let response = ctx
        .client
        .get(&format!("{}/schemas/{}", ctx.base_url, schema.id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let retrieved: Schema = response.json().await.unwrap();
    assert_eq!(retrieved.id, schema.id);
    assert_eq!(retrieved.name, "get-test");
}

#[tokio::test]
async fn retrieves_existing_schema_by_name_and_version() {
    let ctx = TestContext::new().await;
    let schema_response = ctx
        .client
        .post(&format!("{}/schemas", ctx.base_url))
        .json(&valid_schema_payload("get-test-2"))
        .send()
        .await
        .unwrap();

    let schema: Schema = schema_response.json().await.unwrap();

    let response = ctx
        .client
        .get(&format!(
            "{}/schemas/{}/{}",
            ctx.base_url, schema.name, schema.version
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let retrieved: Schema = response.json().await.unwrap();
    assert_eq!(retrieved.id, schema.id);
    assert_eq!(retrieved.name, "get-test-2");
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .get(&format!(
            "{}/schemas/{}",
            ctx.base_url, "7182c4cb-24dc-4142-890c-3c7755ba673e"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let ctx = TestContext::new().await;

    let response = ctx
        .client
        .get(&format!("{}/schemas/{}", ctx.base_url, "not-a-uuid"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
