use std::sync::Arc;

use vault_core::{db::Pool, BlobStore};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub store: Arc<BlobStore>,
}
