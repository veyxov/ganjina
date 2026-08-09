use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Sharded dir for a hash: `<root>/<hash[0:2]>/<hash[2:4]>`.
    fn dir_for(&self, hash: &str) -> PathBuf {
        self.root.join(&hash[0..2]).join(&hash[2..4])
    }

    fn path_for(&self, hash: &str) -> PathBuf {
        self.dir_for(hash).join(hash)
    }

    /// Hashes `bytes`, writes them to a temp file, then atomically renames into
    /// place at the hash-derived path. Returns the hash and whether this was a
    /// new blob (false = already existed, i.e. a duplicate).
    pub async fn store(&self, bytes: &[u8]) -> std::io::Result<(String, bool)> {
        let hash = blake3::hash(bytes).to_hex().to_string();
        let dir = self.dir_for(&hash);
        let final_path = dir.join(&hash);

        if tokio::fs::try_exists(&final_path).await? {
            return Ok((hash, false));
        }

        tokio::fs::create_dir_all(&dir).await?;

        let tmp_path = dir.join(format!("{hash}.tmp"));
        let mut tmp_file = tokio::fs::File::create(&tmp_path).await?;
        tmp_file.write_all(bytes).await?;
        tmp_file.flush().await?;
        drop(tmp_file);

        tokio::fs::rename(&tmp_path, &final_path).await?;
        Ok((hash, true))
    }

    pub fn read_path(&self, hash: &str) -> PathBuf {
        self.path_for(hash)
    }
}

/// True if this looks like a valid hex hash (guards the `/blobs/{hash}` route
/// against path traversal, since the hash is used directly to build a path).
pub fn is_valid_hash(hash: &str) -> bool {
    hash.len() >= 4 && hash.chars().all(|c| c.is_ascii_hexdigit())
}
