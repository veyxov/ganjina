mod dto;
mod error;
mod handlers;
mod process;
mod state;

use std::sync::Arc;

use axum::{
    routing::{delete, get, put},
    Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};
use vault_core::{db, BlobStore};

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Relative to CWD unless overridden — GANJINA_DATA_DIR avoids storage silently
    // moving if the binary gets launched from a different working directory.
    let data_dir = std::env::var("GANJINA_DATA_DIR").unwrap_or_else(|_| "data".into());
    tokio::fs::create_dir_all(&data_dir).await?;
    let data_dir = tokio::fs::canonicalize(&data_dir).await?;
    tracing::info!(data_dir = %data_dir.display(), "using data directory");

    let pool = db::connect(&data_dir.join("vault.sqlite3")).await?;
    let store = Arc::new(BlobStore::new(data_dir.join("blobs")));
    let state = AppState { pool, store };

    let api = Router::new()
        .route("/assets", get(handlers::assets::list).post(handlers::assets::upload))
        .route("/assets/{id}", get(handlers::assets::get).delete(handlers::assets::delete))
        .route("/assets/{id}/metadata", get(handlers::assets::metadata))
        .route("/assets/{id}/adjacent", get(handlers::assets::adjacent))
        .route("/assets/{id}/owner", put(handlers::assets::set_owner))
        .route("/assets/{id}/collections", get(handlers::collections::for_asset))
        .route("/status", get(handlers::assets::status))
        .route("/collections", get(handlers::collections::list).post(handlers::collections::create))
        .route("/collections/{id}", get(handlers::collections::get))
        .route(
            "/collections/{id}/assets",
            get(handlers::collections::list_assets).post(handlers::collections::add_asset),
        )
        .route(
            "/collections/{id}/assets/{asset_id}",
            delete(handlers::collections::remove_asset),
        )
        .route("/owners", get(handlers::owners::list).post(handlers::owners::create));

    let frontend_dir = std::env::var("GANJINA_FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".into());
    let spa = ServeDir::new(&frontend_dir).fallback(tower_http::services::ServeFile::new(
        format!("{frontend_dir}/index.html"),
    ));

    let app = Router::new()
        .route("/blobs/{hash}", get(handlers::assets::serve_blob))
        .nest("/api", api)
        .with_state(state)
        .fallback_service(spa)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
