use crate::common::{fixtures::valid_schema_payload, test_app::TestApp};

pub async fn create_valid_schema(app: &TestApp, name: &str) -> reqwest::Response {
    app.auth()
        .post("/schemas")
        .json(&valid_schema_payload(name))
        .send()
        .await
        .unwrap()
}

pub async fn create_schema(app: &TestApp, payload: &serde_json::Value) -> reqwest::Response {
    app.auth()
        .post("/schemas")
        .json(&payload)
        .send()
        .await
        .unwrap()
}

pub async fn get_schemas(app: &TestApp) -> reqwest::Response {
    app.auth().get("/schemas").send().await.unwrap()
}

pub async fn get_schema_by_id(app: &TestApp, schema_id: &str) -> reqwest::Response {
    app.auth()
        .get(format!("/schemas/{}", schema_id))
        .send()
        .await
        .unwrap()
}

pub async fn get_schema_by_name_and_version(
    app: &TestApp,
    schema_name: &str,
    schema_version: &str,
) -> reqwest::Response {
    app.auth()
        .get(format!(
            "/schemas/by-name/{}/versions/{}",
            schema_name, schema_version
        ))
        .send()
        .await
        .unwrap()
}

pub async fn update_schema(
    app: &TestApp,
    schema_id: &str,
    payload: &serde_json::Value,
) -> reqwest::Response {
    app.auth()
        .put(format!("/schemas/{}", schema_id))
        .json(&payload)
        .send()
        .await
        .unwrap()
}

pub async fn delete_schema(app: &TestApp, schema_id: &str) -> reqwest::Response {
    app.auth()
        .delete(format!("/schemas/{}", schema_id))
        .send()
        .await
        .unwrap()
}

pub async fn get_schemas_with_cursor(
    app: &TestApp,
    cursor: Option<String>,
    limit: i32,
    direction: &str,
) -> reqwest::Response {
    let mut query_params = vec![
        ("limit", limit.to_string()),
        ("direction", direction.to_string()),
    ];

    if let Some(c) = cursor {
        query_params.push(("cursor", c));
    }

    app.auth()
        .get("/schemas")
        .query(&query_params)
        .send()
        .await
        .unwrap()
}
