use axum::{
    extract::{Multipart, Path, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use vault_core::db;

use crate::{dto::*, error::AppError, process::process_asset, state::AppState};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<db::Asset>>, AppError> {
    Ok(Json(db::list_assets(&state.pool).await?))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<db::Asset>, AppError> {
    let asset = db::asset_by_id(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(asset))
}

pub async fn metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Option<db::PhotoMetadata>>, AppError> {
    Ok(Json(db::photo_metadata_for_asset(&state.pool, &id).await?))
}

pub async fn adjacent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AdjacentAssets>, AppError> {
    let (prev_id, next_id) = db::adjacent_asset_ids(&state.pool, &id).await?;
    Ok(Json(AdjacentAssets { prev_id, next_id }))
}

pub async fn status(State(state): State<AppState>) -> Result<Json<StatusSummary>, AppError> {
    let failed_count = db::failed_asset_count(&state.pool).await?;
    Ok(Json(StatusSummary { failed_count }))
}

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<db::Asset>>, AppError> {
    let mut created = Vec::new();

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
            let asset_id = db::insert_asset(
                &state.pool,
                &hash,
                &original_filename,
                bytes.len() as i64,
                &content_type,
            )
            .await?;
            tracing::info!(%hash, original_filename, size_bytes = bytes.len(), "stored new asset");

            tokio::spawn(process_asset(
                state.pool.clone(),
                state.store.clone(),
                asset_id.clone(),
                bytes,
                content_type,
            ));

            if let Some(asset) = db::asset_by_id(&state.pool, &asset_id).await? {
                created.push(asset);
            }
        } else {
            tracing::info!(%hash, original_filename, "duplicate upload, skipped");
        }
    }

    Ok(Json(created))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(), AppError> {
    let Some((hash, thumbnail_hash)) = db::delete_asset(&state.pool, &id).await? else {
        return Err(AppError::NotFound);
    };

    for h in [Some(hash), thumbnail_hash].into_iter().flatten() {
        if !db::hash_still_referenced(&state.pool, &h).await? {
            state.store.delete(&h).await?;
        }
    }

    tracing::info!(asset_id = id, "deleted asset");
    Ok(())
}

pub async fn set_owner(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SetAssetOwner>,
) -> Result<(), AppError> {
    db::set_asset_owner(&state.pool, &id, body.owner_id.as_deref()).await?;
    Ok(())
}

pub async fn serve_blob(
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
