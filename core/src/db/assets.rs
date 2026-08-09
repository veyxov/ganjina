use uuid::Uuid;

use super::Pool;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct Asset {
    pub id: String,
    pub hash: String,
    pub original_filename: String,
    pub size_bytes: i64,
    pub content_type: String,
    pub thumbnail_hash: Option<String>,
    pub owner_id: Option<String>,
    /// Original (not thumbnail) pixel dimensions — null until the thumbnail
    /// job runs. Used for the justified-layout gallery, which needs real
    /// aspect ratios instead of forcing every tile to a square crop.
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) const ASSET_COLUMNS: &str = "id, hash, original_filename, size_bytes, content_type, \
    thumbnail_hash, owner_id, width, height, created_at";

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

pub async fn set_thumbnail(
    pool: &Pool,
    asset_id: &str,
    thumbnail_hash: &str,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE assets SET thumbnail_hash = ?, width = ?, height = ? WHERE id = ?")
        .bind(thumbnail_hash)
        .bind(width as i64)
        .bind(height as i64)
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
    let row = sqlx::query_as::<_, Asset>(&format!("SELECT {ASSET_COLUMNS} FROM assets WHERE id = ?"))
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

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PhotoMetadata {
    pub taken_at_local: Option<chrono::NaiveDateTime>,
    pub taken_at_offset_minutes: Option<i32>,
    pub camera: Option<String>,
    pub gps_lat: Option<f64>,
    pub gps_lon: Option<f64>,
}

pub async fn set_photo_metadata(pool: &Pool, asset_id: &str, m: &PhotoMetadata) -> anyhow::Result<()> {
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

pub async fn photo_metadata_for_asset(pool: &Pool, asset_id: &str) -> anyhow::Result<Option<PhotoMetadata>> {
    let row = sqlx::query_as::<_, PhotoMetadata>(
        "SELECT taken_at_local, taken_at_offset_minutes, camera, gps_lat, gps_lon
         FROM photo_metadata WHERE asset_id = ?",
    )
    .bind(asset_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
