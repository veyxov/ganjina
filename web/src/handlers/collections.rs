use axum::{
    extract::{Path, State},
    Json,
};
use vault_core::db;

use crate::{dto::*, error::AppError, state::AppState};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<db::Collection>>, AppError> {
    Ok(Json(db::list_collections(&state.pool).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateCollection>,
) -> Result<Json<db::Collection>, AppError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("collection name required".into()));
    }
    let id = db::create_collection(&state.pool, name).await?;
    tracing::info!(collection_id = %id, name, "created collection");
    let collection = db::collection_by_id(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(collection))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<db::Collection>, AppError> {
    let collection = db::collection_by_id(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(collection))
}

pub async fn list_assets(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::Asset>>, AppError> {
    Ok(Json(db::list_assets_in_collection(&state.pool, &id).await?))
}

pub async fn add_asset(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddAssetToCollection>,
) -> Result<(), AppError> {
    db::add_asset_to_collection(&state.pool, &id, &body.asset_id).await?;
    tracing::info!(collection_id = id, asset_id = body.asset_id, "added asset to collection");
    Ok(())
}

pub async fn remove_asset(
    State(state): State<AppState>,
    Path((id, asset_id)): Path<(String, String)>,
) -> Result<(), AppError> {
    db::remove_asset_from_collection(&state.pool, &id, &asset_id).await?;
    Ok(())
}

pub async fn for_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
) -> Result<Json<Vec<db::Collection>>, AppError> {
    Ok(Json(db::collections_for_asset(&state.pool, &asset_id).await?))
}
