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
    pub created_at: String,
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
    let created_at = chrono::Utc::now().to_rfc3339();
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
