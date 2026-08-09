use uuid::Uuid;

use super::Pool;

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
