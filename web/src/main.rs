use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use vault_core::{db, BlobStore, Pool};

#[derive(Clone)]
struct AppState {
    pool: Pool,
    store: Arc<BlobStore>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    assets: Vec<db::Asset>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_dir = std::path::Path::new("data");
    tokio::fs::create_dir_all(data_dir).await?;

    let pool = db::connect(&data_dir.join("vault.sqlite3")).await?;
    let store = Arc::new(BlobStore::new(data_dir.join("blobs")));

    let state = AppState { pool, store };

    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload))
        .route("/blobs/{hash}", get(serve_blob))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let assets = db::list_assets(&state.pool).await?;
    let html = IndexTemplate { assets }.render()?;
    Ok(Html(html))
}

async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Redirect, AppError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("file") {
            continue;
        }
        let original_filename = field.file_name().unwrap_or("upload").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = field.bytes().await?;

        let (hash, is_new) = state.store.store(&bytes).await?;
        if is_new {
            db::insert_asset(
                &state.pool,
                &hash,
                &original_filename,
                bytes.len() as i64,
                &content_type,
            )
            .await?;
        }
    }
    Ok(Redirect::to("/"))
}

async fn serve_blob(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Response, AppError> {
    if !vault_core::store::is_valid_hash(&hash) {
        return Err(AppError(anyhow::anyhow!("invalid hash")));
    }
    let path = state.store.read_path(&hash);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| AppError(anyhow::anyhow!("blob not found")))?;

    let content_type = db::content_type_for_hash(&state.pool, &hash)
        .await?
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}

/// Wraps any error into a 500 response — fine for this stage, will get real
/// error handling once there's more than one failure mode to distinguish.
struct AppError(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
