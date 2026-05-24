pub mod error;
pub mod hybrid;
pub mod local;
pub mod s3;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use cid::Cid as CID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOpts {
    pub pin: bool,
    pub ttl: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinPolicy {
    Permanent,
    TTL(std::time::Duration),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedItem {
    pub cid: CID,
    pub policy: PinPolicy,
    pub pinned_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn add(&self, bytes: &[u8], opts: StorageOpts) -> Result<CID>;
    async fn get(&self, cid: &CID) -> Result<Vec<u8>>;
    async fn pin(&self, cid: &CID, policy: PinPolicy) -> Result<()>;
    async fn unpin(&self, cid: &CID) -> Result<()>;
    async fn ls_pins(&self) -> Result<Vec<PinnedItem>>;
    async fn delete(&self, cid: &CID) -> Result<()>;
}

pub type Result<T> = std::result::Result<T, error::StorageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_opts_serde() {
        let opts = StorageOpts {
            pin: true,
            ttl: Some(std::time::Duration::from_secs(3600)),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let deserialized: StorageOpts = serde_json::from_str(&json).unwrap();
        assert!(deserialized.pin);
    }
}
