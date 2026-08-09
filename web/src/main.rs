use axum::{response::Html, routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { Html("") }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("server error");
}
