use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Form, Multipart, Path, State},
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
    collections: Vec<db::Collection>,
    active_collection: Option<String>,
}

#[derive(Template)]
#[template(path = "collection.html")]
struct CollectionTemplate {
    assets: Vec<db::Asset>,
    collections: Vec<db::Collection>,
    active_collection: Option<String>,
    collection_name: String,
}

#[derive(serde::Deserialize)]
struct CreateCollection {
    name: String,
}

#[derive(serde::Deserialize)]
struct AddAssetToCollection {
    asset_id: String,
    collection_id: String,
}

#[derive(Template)]
#[template(path = "asset_detail.html")]
struct AssetDetailTemplate {
    asset: db::Asset,
    collections: Vec<db::Collection>,
    active_collection: Option<String>,
    in_collections: Vec<db::Collection>,
    available_collections: Vec<db::Collection>,
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
        .route("/collections", post(create_collection))
        .route("/collections/{id}", get(view_collection))
        .route("/collections/add-asset", post(add_asset_to_collection))
        .route("/collections/remove-asset", post(remove_asset_from_collection))
        .route("/assets/{id}", get(view_asset))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let assets = db::list_assets(&state.pool).await?;
    let collections = db::list_collections(&state.pool).await?;
    let html = IndexTemplate {
        assets,
        collections,
        active_collection: None,
    }
    .render()?;
    Ok(Html(html))
}

async fn view_collection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, AppError> {
    let collection = db::collection_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    let assets = db::list_assets_in_collection(&state.pool, &id).await?;
    let collections = db::list_collections(&state.pool).await?;
    let html = CollectionTemplate {
        assets,
        collections,
        active_collection: Some(id),
        collection_name: collection.name,
    }
    .render()?;
    Ok(Html(html))
}

async fn create_collection(
    State(state): State<AppState>,
    Form(form): Form<CreateCollection>,
) -> Result<Redirect, AppError> {
    let name = form.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("collection name required".into()));
    }
    let id = db::create_collection(&state.pool, name).await?;
    tracing::info!(collection_id = %id, name, "created collection");
    Ok(Redirect::to(&format!("/collections/{id}")))
}

async fn add_asset_to_collection(
    State(state): State<AppState>,
    Form(form): Form<AddAssetToCollection>,
) -> Result<Redirect, AppError> {
    db::add_asset_to_collection(&state.pool, &form.collection_id, &form.asset_id).await?;
    tracing::info!(
        collection_id = form.collection_id,
        asset_id = form.asset_id,
        "added asset to collection"
    );
    Ok(Redirect::to("/"))
}

async fn view_asset(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Html<String>, AppError> {
    let asset = db::asset_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    let all_collections = db::list_collections(&state.pool).await?;
    let in_collections = db::collections_for_asset(&state.pool, &id).await?;
    let in_ids: std::collections::HashSet<_> = in_collections.iter().map(|c| &c.id).collect();
    let available_collections: Vec<db::Collection> = all_collections
        .iter()
        .filter(|c| !in_ids.contains(&c.id))
        .cloned()
        .collect();

    let html = AssetDetailTemplate {
        asset,
        collections: all_collections,
        active_collection: None,
        in_collections,
        available_collections,
    }
    .render()?;
    Ok(Html(html))
}

async fn remove_asset_from_collection(
    State(state): State<AppState>,
    Form(form): Form<AddAssetToCollection>,
) -> Result<Redirect, AppError> {
    db::remove_asset_from_collection(&state.pool, &form.collection_id, &form.asset_id).await?;
    Ok(Redirect::to(&format!("/assets/{}", form.asset_id)))
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
