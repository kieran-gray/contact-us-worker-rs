use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::warn;
use worker::kv::KvStore;

#[derive(Debug, Clone)]
pub enum CacheError {
    WriteError(String),
    ReadError(String),
    DeleteError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::WriteError(msg) => write!(f, "Write error: {msg}"),
            CacheError::ReadError(msg) => write!(f, "Read error: {msg}"),
            CacheError::DeleteError(msg) => write!(f, "Delete error: {msg}"),
        }
    }
}

impl std::error::Error for CacheError {}

#[async_trait(?Send)]
pub trait CacheTrait {
    async fn set<T: Serialize>(&self, key: String, value: T) -> Result<(), CacheError>;
    async fn get<T: for<'de> Deserialize<'de>>(&self, key: String)
    -> Result<Option<T>, CacheError>;
    async fn clear(&self, key: String) -> Result<(), CacheError>;
}

pub struct KVCache {
    cache: KvStore,
    ttl: u64,
}

impl KVCache {
    pub fn create(cache: KvStore, ttl: u64) -> Self {
        Self { cache, ttl }
    }
}

#[async_trait(?Send)]
impl CacheTrait for KVCache {
    async fn set<T: Serialize>(&self, key: String, value: T) -> Result<(), CacheError> {
        self.cache
            .put(&key, value)
            .unwrap()
            .expiration_ttl(self.ttl)
            .execute()
            .await
            .map_err(|e| {
                let err = format!("Failed to add to cache: {e}");
                warn!(
                    error = %e,
                    cache_key = %key,
                    "Cache write failed"
                );
                CacheError::WriteError(err)
            })
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, id: String) -> Result<Option<T>, CacheError> {
        let cache_result = self.cache.get(&id).json::<T>().await.map_err(|e| {
            let err = format!("Failed to fetch from cache: {e}");
            warn!(
                error = %e,
                cache_key = %id,
                "Cache read failed"
            );
            CacheError::ReadError(err)
        })?;
        Ok(cache_result)
    }

    async fn clear(&self, id: String) -> Result<(), CacheError> {
        self.cache.delete(&id).await.map_err(|e| {
            let err = format!("Failed to delete from cache: {e}");
            warn!(
                error = %e,
                cache_key = %id,
                "Cache delete failed"
            );
            CacheError::DeleteError(err)
        })?;
        Ok(())
    }
}
