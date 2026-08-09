use uuid::Uuid;

use super::{Asset, Pool, ASSET_COLUMNS};

#[derive(sqlx::FromRow, Clone, serde::Serialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_collection(pool: &Pool, name: &str) -> anyhow::Result<String> {
    let id = Uuid::now_v7().to_string();
    sqlx::query("INSERT INTO collections (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(chrono::Utc::now())
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn list_collections(pool: &Pool) -> anyhow::Result<Vec<Collection>> {
    let rows = sqlx::query_as::<_, Collection>("SELECT id, name, created_at FROM collections ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn collection_by_id(pool: &Pool, id: &str) -> anyhow::Result<Option<Collection>> {
    let row =
        sqlx::query_as::<_, Collection>("SELECT id, name, created_at FROM collections WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

pub async fn add_asset_to_collection(pool: &Pool, collection_id: &str, asset_id: &str) -> anyhow::Result<()> {
    sqlx::query("INSERT OR IGNORE INTO collection_assets (collection_id, asset_id) VALUES (?, ?)")
        .bind(collection_id)
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_assets_in_collection(pool: &Pool, collection_id: &str) -> anyhow::Result<Vec<Asset>> {
    let rows = sqlx::query_as::<_, Asset>(&format!(
        "SELECT {ASSET_COLUMNS} FROM assets a
         JOIN collection_assets ca ON ca.asset_id = a.id
         WHERE ca.collection_id = ?
         ORDER BY a.created_at DESC"
    ))
    .bind(collection_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn collections_for_asset(pool: &Pool, asset_id: &str) -> anyhow::Result<Vec<Collection>> {
    let rows = sqlx::query_as::<_, Collection>(
        "SELECT c.id, c.name, c.created_at
         FROM collections c
         JOIN collection_assets ca ON ca.collection_id = c.id
         WHERE ca.asset_id = ?
         ORDER BY c.name",
    )
    .bind(asset_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn remove_asset_from_collection(pool: &Pool, collection_id: &str, asset_id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM collection_assets WHERE collection_id = ? AND asset_id = ?")
        .bind(collection_id)
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}
