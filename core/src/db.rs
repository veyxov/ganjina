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
    pub thumbnail_hash: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

const ASSET_COLUMNS: &str =
    "id, hash, original_filename, size_bytes, content_type, thumbnail_hash, owner_id, created_at";

/// Inserts a new asset row for a freshly stored blob. Caller is responsible for
/// only calling this when `BlobStore::store` reported a new (non-duplicate) blob.
/// Returns the new asset's id so the caller can enqueue processing for it.
pub async fn insert_asset(
    pool: &Pool,
    hash: &str,
    original_filename: &str,
    size_bytes: i64,
    content_type: &str,
) -> anyhow::Result<String> {
    let id = Uuid::now_v7().to_string();
    let created_at = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO assets (id, hash, original_filename, size_bytes, content_type, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(hash)
    .bind(original_filename)
    .bind(size_bytes)
    .bind(content_type)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn set_thumbnail_hash(pool: &Pool, asset_id: &str, thumbnail_hash: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE assets SET thumbnail_hash = ? WHERE id = ?")
        .bind(thumbnail_hash)
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Thumbnails are always stored as JPEG (see `photos::generate_thumbnail`), so a
/// hash matching an asset's `thumbnail_hash` gets a hardcoded content type
/// rather than a lookup — thumbnails aren't assets with their own row.
pub async fn content_type_for_hash(pool: &Pool, hash: &str) -> anyhow::Result<Option<String>> {
    let row = sqlx::query_scalar::<_, String>(
        "SELECT content_type FROM assets WHERE hash = ?
         UNION ALL
         SELECT 'image/jpeg' FROM assets WHERE thumbnail_hash = ?
         LIMIT 1",
    )
    .bind(hash)
    .bind(hash)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn asset_by_id(pool: &Pool, id: &str) -> anyhow::Result<Option<Asset>> {
    let row = sqlx::query_as::<_, Asset>(&format!(
        "SELECT {ASSET_COLUMNS} FROM assets WHERE id = ?"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_assets(pool: &Pool) -> anyhow::Result<Vec<Asset>> {
    let rows = sqlx::query_as::<_, Asset>(&format!(
        "SELECT {ASSET_COLUMNS} FROM assets ORDER BY created_at DESC"
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Deletes the asset row (and its collection memberships — hash is UNIQUE per
/// asset, so no other row can reference the same blob) and returns the hashes
/// so the caller can remove the now-unreferenced blob files.
pub async fn delete_asset(pool: &Pool, id: &str) -> anyhow::Result<Option<(String, Option<String>)>> {
    let asset = asset_by_id(pool, id).await?;
    let Some(asset) = asset else { return Ok(None) };

    sqlx::query("DELETE FROM collection_assets WHERE asset_id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM photo_metadata WHERE asset_id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM jobs WHERE asset_id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM asset_links WHERE asset_a = ? OR asset_b = ?")
        .bind(id)
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM assets WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(Some((asset.hash, asset.thumbnail_hash)))
}

#[derive(sqlx::FromRow, Clone)]
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

/// True if any asset still points at this hash as its original or thumbnail —
/// content-addressing means two different assets can share a hash (e.g.
/// identical-after-resize thumbnails), so a blob is only safe to delete once
/// nothing references it any more.
pub async fn hash_still_referenced(pool: &Pool, hash: &str) -> anyhow::Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM assets WHERE hash = ? OR thumbnail_hash = ?",
    )
    .bind(hash)
    .bind(hash)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

#[derive(sqlx::FromRow, Clone)]
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

pub async fn remove_asset_from_collection(
    pool: &Pool,
    collection_id: &str,
    asset_id: &str,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM collection_assets WHERE collection_id = ? AND asset_id = ?")
        .bind(collection_id)
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
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

#[derive(sqlx::FromRow)]
pub struct PhotoMetadata {
    pub taken_at_local: Option<chrono::NaiveDateTime>,
    pub taken_at_offset_minutes: Option<i32>,
    pub camera: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

pub async fn set_photo_metadata(
    pool: &Pool,
    asset_id: &str,
    m: &PhotoMetadata,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO photo_metadata
            (asset_id, taken_at_local, taken_at_offset_minutes, camera, gps_lat, gps_lon)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(asset_id) DO UPDATE SET
            taken_at_local = excluded.taken_at_local,
            taken_at_offset_minutes = excluded.taken_at_offset_minutes,
            camera = excluded.camera,
            gps_lat = excluded.gps_lat,
            gps_lon = excluded.gps_lon",
    )
    .bind(asset_id)
    .bind(m.taken_at_local)
    .bind(m.taken_at_offset_minutes)
    .bind(&m.camera)
    .bind(m.gps_lat)
    .bind(m.gps_lon)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn photo_metadata_for_asset(
    pool: &Pool,
    asset_id: &str,
) -> anyhow::Result<Option<PhotoMetadata>> {
    let row = sqlx::query_as::<_, PhotoMetadata>(
        "SELECT taken_at_local, taken_at_offset_minutes, camera, gps_lat, gps_lon
         FROM photo_metadata WHERE asset_id = ?",
    )
    .bind(asset_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Records one processing attempt for an asset (thumbnail/EXIF extraction). An
/// append-only log rather than an in-place-updated row — simplest thing that
/// gives durability/observability for the job pipeline without a state machine.
pub async fn record_job(
    pool: &Pool,
    asset_id: &str,
    job_type: &str,
    status: &str,
    error: Option<&str>,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO jobs (id, asset_id, job_type, status, error, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(asset_id)
    .bind(job_type)
    .bind(status)
    .bind(error)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Neighboring asset ids in the same order as `list_assets` (newest first), for
/// prev/next navigation in the detail view.
pub async fn adjacent_asset_ids(
    pool: &Pool,
    asset_id: &str,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    let prev = sqlx::query_scalar::<_, String>(
        "SELECT id FROM assets WHERE created_at > (SELECT created_at FROM assets WHERE id = ?)
         ORDER BY created_at ASC LIMIT 1",
    )
    .bind(asset_id)
    .fetch_optional(pool)
    .await?;
    let next = sqlx::query_scalar::<_, String>(
        "SELECT id FROM assets WHERE created_at < (SELECT created_at FROM assets WHERE id = ?)
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(asset_id)
    .fetch_optional(pool)
    .await?;
    Ok((prev, next))
}

/// Assets whose most recent job attempt failed — surfaced in the UI so a
/// failure never gets silently lost.
pub async fn failed_asset_count(pool: &Pool) -> anyhow::Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT asset_id) FROM jobs j1
         WHERE status = 'failed' AND NOT EXISTS (
            SELECT 1 FROM jobs j2
            WHERE j2.asset_id = j1.asset_id AND j2.job_type = j1.job_type
              AND j2.created_at > j1.created_at
         )",
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}
