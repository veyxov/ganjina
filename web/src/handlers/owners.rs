use axum::{extract::State, Json};
use vault_core::db;

use crate::{dto::CreateOwner, error::AppError, state::AppState};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<db::Owner>>, AppError> {
    Ok(Json(db::list_owners(&state.pool).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateOwner>,
) -> Result<Json<db::Owner>, AppError> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("owner name required".into()));
    }
    let id = db::create_owner(&state.pool, name).await?;
    Ok(Json(db::Owner { id, name: name.to_string() }))
}
