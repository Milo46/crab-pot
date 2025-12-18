use log_server::Schema;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::common::{fixtures::valid_schema_payload, test_app::setup_test_app};

#[tokio::test]
async fn creates_schema_with_valid_data() {
    let app = setup_test_app().await;

    let response = app
        .client
        .post(&format!("{}/schemas", app.address))
        .header("X-Api-Key", "secret-key")
        .json(&valid_schema_payload("test-schema"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);

    let schema: Schema = response.json().await.unwrap();
    assert_eq!(schema.name, "test-schema");
    assert_eq!(schema.version, "1.0.0");
    let uuid_str = schema.id.to_string();
    assert!(
        Uuid::parse_str(&uuid_str).is_ok(),
        "Schema ID should be a valid UUID"
    );
    assert!(schema.created_at.timestamp() > 0);
}

#[tokio::test]
async fn returns_201_with_location_header() {
    let app = setup_test_app().await;

    let response = app
        .client
        .post(&format!("{}/schemas", app.address))
        .header("X-Api-Key", "secret-key")
        .json(&valid_schema_payload("location-test"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let location = response
        .headers()
        .get("Location")
        .expect("Location header should be present");

    assert!(location.to_str().unwrap().contains("/schemas/"));
}
