use uuid::Uuid;

use super::Pool;

#[derive(sqlx::FromRow, Clone, serde::Serialize)]
pub struct Owner {
    pub id: String,
    pub name: String,
}

pub async fn create_owner(pool: &Pool, name: &str) -> anyhow::Result<String> {
    let id = Uuid::now_v7().to_string();
    sqlx::query("INSERT INTO owners (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(name)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn list_owners(pool: &Pool) -> anyhow::Result<Vec<Owner>> {
    let rows = sqlx::query_as::<_, Owner>("SELECT id, name FROM owners ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn set_asset_owner(pool: &Pool, asset_id: &str, owner_id: Option<&str>) -> anyhow::Result<()> {
    sqlx::query("UPDATE assets SET owner_id = ? WHERE id = ?")
        .bind(owner_id)
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}
