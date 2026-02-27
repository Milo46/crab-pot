use crab_pot::Schema;
use reqwest::StatusCode;

use crate::common::{
    routes::schemas::{create_valid_schema, get_schema_by_id, get_schema_by_name_and_version},
    test_app::setup_test_app,
};

#[tokio::test]
async fn retrieves_existing_schema_by_id() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "get-test").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let response = get_schema_by_id(&app, &schema.id.to_string()).await;
    assert_eq!(response.status(), StatusCode::OK);

    let retrieved: Schema = response.json().await.unwrap();
    assert_eq!(retrieved.id, schema.id);
    assert_eq!(retrieved.name, "get-test");
}

#[tokio::test]
async fn retrieves_existing_schema_by_name_and_version() {
    let app = setup_test_app().await;

    let schema_response = create_valid_schema(&app, "get-test-2").await;
    let schema: Schema = schema_response.json().await.unwrap();

    let response = get_schema_by_name_and_version(&app, &schema.name, &schema.version).await;
    assert_eq!(response.status(), StatusCode::OK);

    let retrieved: Schema = response.json().await.unwrap();
    assert_eq!(retrieved.id, schema.id);
    assert_eq!(retrieved.name, "get-test-2");
}

#[tokio::test]
async fn returns_404_for_nonexistent_schema() {
    let app = setup_test_app().await;

    let response = get_schema_by_id(&app, "7182c4cb-24dc-4142-890c-3c7755ba673e").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rejects_invalid_uuid_format() {
    let app = setup_test_app().await;

    let response = get_schema_by_id(&app, "not-a-uuid").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
