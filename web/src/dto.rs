use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateCollection {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AddAssetToCollection {
    pub asset_id: String,
}

#[derive(Deserialize)]
pub struct CreateOwner {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SetAssetOwner {
    /// `null` (or omitted) unassigns.
    pub owner_id: Option<String>,
}

#[derive(Serialize)]
pub struct AdjacentAssets {
    pub prev_id: Option<String>,
    pub next_id: Option<String>,
}

#[derive(Serialize)]
pub struct StatusSummary {
    pub failed_count: i64,
}
