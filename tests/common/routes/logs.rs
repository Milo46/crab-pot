use crate::common::{
    fixtures::{valid_log_payload, valid_log_payload_with_message},
    test_app::TestApp,
};

pub async fn create_log(app: &TestApp, payload: &serde_json::Value) -> reqwest::Response {
    app.auth()
        .post("/logs")
        .json(&payload)
        .send()
        .await
        .unwrap()
}

pub async fn create_valid_log<S: AsRef<str>>(app: &TestApp, schema_id: S) -> reqwest::Response {
    app.auth()
        .post("/logs")
        .json(&valid_log_payload(schema_id.as_ref()))
        .send()
        .await
        .unwrap()
}

pub async fn create_valid_log_with_message<S: AsRef<str>>(
    app: &TestApp,
    schema_id: S,
    message: &str,
) -> reqwest::Response {
    app.auth()
        .post("/logs")
        .json(&valid_log_payload_with_message(schema_id.as_ref(), message))
        .send()
        .await
        .unwrap()
}

pub async fn get_log<S: AsRef<str>>(app: &TestApp, id: S) -> reqwest::Response {
    app.auth()
        .get(format!("/logs/{}", id.as_ref()))
        .send()
        .await
        .unwrap()
}

pub async fn get_logs_by_schema_name<S: AsRef<str>>(
    app: &TestApp,
    schema_name: S,
) -> reqwest::Response {
    app.auth()
        .get(format!("/logs/schema/{}", schema_name.as_ref()))
        .send()
        .await
        .unwrap()
}

pub async fn get_logs_by_schema_name_and_version<S: AsRef<str>>(
    app: &TestApp,
    schema_name: S,
    schema_version: S,
) -> reqwest::Response {
    app.auth()
        .get(format!(
            "/logs/schema/{}/versions/{}",
            schema_name.as_ref(),
            schema_version.as_ref()
        ))
        .send()
        .await
        .unwrap()
}

pub async fn delete_log<S: AsRef<str>>(app: &TestApp, id: S) -> reqwest::Response {
    app.auth()
        .delete(format!("/logs/{}", id.as_ref()))
        .send()
        .await
        .unwrap()
}
