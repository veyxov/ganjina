mod assets;
mod collections;
mod jobs;
mod owners;

pub use assets::*;
pub use collections::*;
pub use jobs::*;
pub use owners::*;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

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
