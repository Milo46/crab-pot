use crate::common::test_app::AdminTestApp;

pub async fn get_api_keys(app: &AdminTestApp) -> reqwest::Response {
    app.client().get("/api-keys").send().await.unwrap()
}

pub async fn get_api_key_by_id(app: &AdminTestApp, key_id: i32) -> reqwest::Response {
    app.client()
        .get(format!("/api-keys/{}", key_id))
        .send()
        .await
        .unwrap()
}

pub async fn create_api_key(app: &AdminTestApp, payload: &serde_json::Value) -> reqwest::Response {
    app.client()
        .post("/api-keys")
        .json(&payload)
        .send()
        .await
        .unwrap()
}

pub async fn delete_api_key(app: &AdminTestApp, key_id: i32) -> reqwest::Response {
    app.client()
        .delete(format!("/api-keys/{}", key_id))
        .send()
        .await
        .unwrap()
}

pub async fn rotate_api_key(app: &AdminTestApp, key_id: i32) -> reqwest::Response {
    app.client()
        .post(format!("/api-keys/{}/rotate", key_id))
        .send()
        .await
        .unwrap()
}
