use axum::{
    routing::get,
    extract::Path,
    Router,
    Json,
};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/user/{id}", get(user_handler));
    
    println!("Axum server running at http://0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello from /" }))
}

async fn user_handler(Path(id): Path<u32>) -> Json<serde_json::Value> {
    Json(json!({
        "message": format!("Hello user with id {}", id)
    }))
}
