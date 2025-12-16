use reqwest::StatusCode;

use crate::common::{
    routes::schemas::{create_valid_schema, get_schemas},
    test_app::setup_test_app,
};

#[tokio::test]
async fn lists_all_schemas() {
    let app = setup_test_app().await;

    let initial_response = get_schemas(&app).await;
    let initial_data: serde_json::Value = initial_response.json().await.unwrap();
    let initial_count = initial_data["schemas"].as_array().unwrap().len();

    let _ = create_valid_schema(&app, "list-test-1").await;
    let _ = create_valid_schema(&app, "list-test-2").await;

    let response = get_schemas(&app).await;
    assert_eq!(response.status(), StatusCode::OK);

    let data: serde_json::Value = response.json().await.unwrap();
    let schemas = data["schemas"].as_array().unwrap();
    assert_eq!(
        schemas.len(),
        initial_count + 2,
        "Expected {} schemas (initial {} = 2 new), but got {}",
        initial_count + 2,
        initial_count,
        schemas.len()
    );

    let schemas_names: Vec<&str> = schemas.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(schemas_names.contains(&"list-test-1"));
    assert!(schemas_names.contains(&"list-test-2"));
}
