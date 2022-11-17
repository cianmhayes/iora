use crate::{
    Asset, AssetCache, AssetCatalog, AssetDescriptor, FetchAssetsError, ListAssetsError,
    NameConstraint, UpdateCacheError, VersionConstraint,
};
pub trait AssetCatalogWithCache: AssetCatalog + AssetCache {}

pub struct CachingAssetCatalog {
    cache: Box<dyn AssetCatalogWithCache>,
    remote_catalog: Box<dyn AssetCatalog>,
}

impl CachingAssetCatalog {
    pub fn new(cache: Box<dyn AssetCatalogWithCache>, remote: Box<dyn AssetCatalog>) -> Self {
        CachingAssetCatalog {
            remote_catalog: remote,
            cache: cache,
        }
    }
}

impl AssetCatalog for CachingAssetCatalog {
    fn fetch_asset(&self, asset_descriptor: &AssetDescriptor) -> Result<Asset, FetchAssetsError> {
        let cached_result = self.cache.fetch_asset(asset_descriptor);
        if let Ok(asset) = cached_result {
            return Ok(asset);
        }
        match self.remote_catalog.fetch_asset(asset_descriptor) {
            Ok(asset) => {
                self.save_asset(&asset);
                Ok(asset)
            }
            Err(e) => Err(e),
        }
    }

    fn list_assets(
        &self,
        name_constraint: &NameConstraint,
        version_constraint: &Option<VersionConstraint>,
    ) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        let cached_result = self.cache.list_assets(name_constraint, version_constraint);
        if let Ok(list) = cached_result {
            return Ok(list);
        }
        match self
            .remote_catalog
            .list_assets(name_constraint, version_constraint)
        {
            Ok(list) => {
                self.save_descriptors(&list);
                Ok(list)
            }
            Err(e) => Err(e),
        }
    }
}

impl AssetCache for CachingAssetCatalog {
    fn save_asset(&self, descriptor: &Asset) -> Result<(), UpdateCacheError> {
        self.cache.save_asset(descriptor)
    }

    fn save_descriptors(&self, descriptor: &Vec<AssetDescriptor>) -> Result<(), UpdateCacheError> {
        self.cache.save_descriptors(descriptor)
    }
}
