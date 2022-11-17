use crate::{AssetDescriptor, AssetQuery};

#[derive(Debug)]
pub enum ListAssetsError {
    NoResults,
    InvalidNameConstraint,
    QueryFailed,
}

pub trait AssetCatalog {
    fn list_assets(
        &self,
        query: &AssetQuery,
    ) -> Result<Vec<AssetDescriptor>, ListAssetsError>;
}

#[derive(Debug)]
pub enum ListAssetsCacheError {
    StorageError,
}

pub trait ListAssetsCache {
    fn save(
        &self,
        descriptor: &Vec<AssetDescriptor>,
        query: &AssetQuery,
    );
}

pub struct CachingAssetCatalog<TCache, TRemote>
where
    TCache: AssetCatalog + ListAssetsCache,
    TRemote: AssetCatalog,
{
    cache: Box<TCache>,
    remote_catalog: Box<TRemote>,
}

impl<TCache, TRemote> CachingAssetCatalog<TCache, TRemote>
where
    TCache: AssetCatalog + ListAssetsCache,
    TRemote: AssetCatalog,
{
    pub fn new(cache: Box<TCache>, remote: Box<TRemote>) -> Self {
        CachingAssetCatalog {
            remote_catalog: remote,
            cache: cache,
        }
    }
}

impl<TCache, TRemote> AssetCatalog for CachingAssetCatalog<TCache, TRemote>
where
    TCache: AssetCatalog + ListAssetsCache,
    TRemote: AssetCatalog,
{
    fn list_assets(
        &self,
        query: &AssetQuery,
    ) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        let cached_result = self.cache.list_assets(query);
        if let Ok(list) = cached_result {
            return Ok(list);
        }
        match self
            .remote_catalog
            .list_assets(query)
        {
            Ok(list) => {
                self.cache.save(&list, query);
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }
}
