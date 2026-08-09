use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use uuid::Uuid;

pub type Pool = SqlitePool;

pub async fn connect(path: &std::path::Path) -> anyhow::Result<Pool> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(sqlx::FromRow)]
pub struct Asset {
    pub id: String,
    pub hash: String,
    pub original_filename: String,
    pub size_bytes: i64,
    pub content_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Inserts a new asset row for a freshly stored blob. Caller is responsible for
/// only calling this when `BlobStore::store` reported a new (non-duplicate) blob.
pub async fn insert_asset(
    pool: &Pool,
    hash: &str,
    original_filename: &str,
    size_bytes: i64,
    content_type: &str,
) -> anyhow::Result<()> {
    let id = Uuid::now_v7().to_string();
    let created_at = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO assets (id, hash, original_filename, size_bytes, content_type, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(hash)
    .bind(original_filename)
    .bind(size_bytes)
    .bind(content_type)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn content_type_for_hash(pool: &Pool, hash: &str) -> anyhow::Result<Option<String>> {
    let row = sqlx::query_scalar::<_, String>("SELECT content_type FROM assets WHERE hash = ?")
        .bind(hash)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_assets(pool: &Pool) -> anyhow::Result<Vec<Asset>> {
    let rows = sqlx::query_as::<_, Asset>(
        "SELECT id, hash, original_filename, size_bytes, content_type, created_at
         FROM assets ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(sqlx::FromRow)]
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
    let rows = sqlx::query_as::<_, Collection>(
        "SELECT id, name, created_at FROM collections ORDER BY name",
    )
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

pub async fn add_asset_to_collection(
    pool: &Pool,
    collection_id: &str,
    asset_id: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO collection_assets (collection_id, asset_id) VALUES (?, ?)",
    )
    .bind(collection_id)
    .bind(asset_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_assets_in_collection(pool: &Pool, collection_id: &str) -> anyhow::Result<Vec<Asset>> {
    let rows = sqlx::query_as::<_, Asset>(
        "SELECT a.id, a.hash, a.original_filename, a.size_bytes, a.content_type, a.created_at
         FROM assets a
         JOIN collection_assets ca ON ca.asset_id = a.id
         WHERE ca.collection_id = ?
         ORDER BY a.created_at DESC",
    )
    .bind(collection_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Links two assets of any type (e.g. a note and the photo it's about). Stores
/// the pair in canonical order so (a, b) and (b, a) collapse to one link. No
/// callers yet — schema/API groundwork for the notes module, ready without a
/// future migration.
#[allow(dead_code)]
pub async fn link_assets(pool: &Pool, asset_a: &str, asset_b: &str) -> anyhow::Result<()> {
    let (a, b) = if asset_a <= asset_b {
        (asset_a, asset_b)
    } else {
        (asset_b, asset_a)
    };
    sqlx::query(
        "INSERT OR IGNORE INTO asset_links (id, asset_a, asset_b, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(a)
    .bind(b)
    .bind(chrono::Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}
