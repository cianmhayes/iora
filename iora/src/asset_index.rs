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

#[cfg(test)]
mod tests {
    use crate::{
        AssetDescriptor, AssetIndex, AssetQuery, CachingAssetIndex, MemoryAssetIndex,
        MemoryAssetIndexCache, NameConstraint, SemVer,
    };
    use std::str::FromStr;
    use std::time::Duration;

    #[test]
    fn list() {
        let cache = Box::new(MemoryAssetIndexCache::new(Duration::from_secs(1)));
        let remote = Box::new(MemoryAssetIndex::default());
        remote.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_name",
            &SemVer::from_str("2.45.6").unwrap(),
            "asset_hash",
        ));
        remote.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_name",
            &SemVer::from_str("3.45.6").unwrap(),
            "asset_hash",
        ));
        remote.descriptors.borrow_mut().push(AssetDescriptor::new(
            "asset_name",
            &SemVer::from_str("3.45.7").unwrap(),
            "asset_hash",
        ));
        let catalog = CachingAssetIndex::new(cache, remote);

        let result = catalog
            .list_assets(&AssetQuery::from((
                &NameConstraint::Contains("asset".to_string()),
                &None,
            )))
            .expect("t");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, SemVer::from_str("3.45.7").unwrap());
    }
}
