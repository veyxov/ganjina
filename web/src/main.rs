use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
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

    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload))
        .route("/blobs/{hash}", get(serve_blob))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("listening on http://127.0.0.1:3000");
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
            tracing::info!(%hash, original_filename, size_bytes = bytes.len(), "stored new asset");
        } else {
            tracing::info!(%hash, original_filename, "duplicate upload, skipped");
        }
    }
    Ok(Redirect::to("/"))
}

async fn serve_blob(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Response, AppError> {
    if !vault_core::store::is_valid_hash(&hash) {
        return Err(AppError::BadRequest("invalid hash".into()));
    }
    let path = state.store.read_path(&hash);
    let open_file = async {
        tokio::fs::File::open(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound
            } else {
                AppError::from(e)
            }
        })
    };
    let lookup_content_type = async {
        db::content_type_for_hash(&state.pool, &hash)
            .await
            .map_err(AppError::from)
    };
    let (file, content_type) = tokio::try_join!(open_file, lookup_content_type)?;

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let body = axum::body::Body::from_stream(tokio_util::io::ReaderStream::new(file));

    Ok(([(header::CONTENT_TYPE, content_type)], body).into_response())
}

enum AppError {
    NotFound,
    BadRequest(String),
    Internal(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
        }
    }
}
