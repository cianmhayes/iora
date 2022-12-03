use crate::{AssetDescriptor, AssetQuery};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListAssetsError {
    #[error("Asset index is missing or unavailable.")]
    AssetIndexNotFound(Option<String>),
    #[error("Asset index refused access.")]
    AssetIndexAccessDenied(Option<String>),
    #[error("Failed to execute the query. Details: {0}")]
    AssetIndexInternalError(String),
    #[error("Failed to execute the query. Details: {details:?}. Query: {query:?}")]
    BadQuery { query: String, details: String },
}

#[derive(Error, Debug)]
pub enum ListAssetsCacheError {
    #[error("Something went wrong in the cache.")]
    StorageError,
}

pub trait AssetIndex {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError>;
}

pub trait ListAssetsCache {
    fn has_cache_entry(&self, query: &AssetQuery) -> bool;
    fn save(&self, descriptor: &[AssetDescriptor], query: &AssetQuery);
}

pub struct CachingAssetIndex<TCache, TRemote>
where
    TCache: AssetIndex + ListAssetsCache,
    TRemote: AssetIndex,
{
    cache: Box<TCache>,
    remote: Box<TRemote>,
}

impl<TCache, TRemote> CachingAssetIndex<TCache, TRemote>
where
    TCache: AssetIndex + ListAssetsCache,
    TRemote: AssetIndex,
{
    pub fn new(cache: Box<TCache>, remote: Box<TRemote>) -> Self {
        CachingAssetIndex { cache, remote }
    }
}

impl<TCache, TRemote> AssetIndex for CachingAssetIndex<TCache, TRemote>
where
    TCache: AssetIndex + ListAssetsCache,
    TRemote: AssetIndex,
{
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        if self.cache.has_cache_entry(query) {
            let cached_result = self.cache.list_assets(query);
            if let Ok(list) = cached_result {
                return Ok(list);
            }
        }
        match self.remote.list_assets(query) {
            Ok(list) => {
                self.cache.save(&list, query);
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }
}
