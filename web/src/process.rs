use std::sync::Arc;

use vault_core::{db, BlobStore, db::Pool};

/// Runs off the request path — upload returns immediately, EXIF/thumbnail land
/// whenever they're done. Failures are logged to the jobs table, never lose the
/// already-stored asset.
pub async fn process_asset(
    pool: Pool,
    store: Arc<BlobStore>,
    asset_id: String,
    bytes: axum::body::Bytes,
    content_type: String,
) {
    if let Some(exif) = photos::extract_exif(&bytes, &content_type) {
        let m = db::PhotoMetadata {
            taken_at_local: exif.taken_at_local,
            taken_at_offset_minutes: exif.taken_at_offset_minutes,
            camera: exif.camera,
            gps_lat: exif.gps_lat,
            gps_lon: exif.gps_lon,
        };
        match db::set_photo_metadata(&pool, &asset_id, &m).await {
            Ok(()) => {
                let _ = db::record_job(&pool, &asset_id, "extract_metadata", "success", None).await;
            }
            Err(e) => {
                tracing::error!(asset_id, error = %e, "failed to save exif metadata");
                let _ = db::record_job(&pool, &asset_id, "extract_metadata", "failed", Some(&e.to_string())).await;
            }
        }
    }

    match photos::generate_thumbnail(bytes.to_vec(), content_type).await {
        Ok(thumb_bytes) => match store.store(&thumb_bytes).await {
            Ok((thumb_hash, _)) => match db::set_thumbnail_hash(&pool, &asset_id, &thumb_hash).await {
                Ok(()) => {
                    let _ = db::record_job(&pool, &asset_id, "thumbnail", "success", None).await;
                    tracing::info!(asset_id, "thumbnail ready");
                }
                Err(e) => {
                    tracing::error!(asset_id, error = %e, "failed to save thumbnail hash");
                    let _ = db::record_job(&pool, &asset_id, "thumbnail", "failed", Some(&e.to_string())).await;
                }
            },
            Err(e) => {
                tracing::error!(asset_id, error = %e, "failed to store thumbnail blob");
                let _ = db::record_job(&pool, &asset_id, "thumbnail", "failed", Some(&e.to_string())).await;
            }
        },
        Err(e) => {
            tracing::error!(asset_id, error = %e, "failed to generate thumbnail");
            let _ = db::record_job(&pool, &asset_id, "thumbnail", "failed", Some(&e.to_string())).await;
        }
    }
}
