use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::{AssetCatalog, AssetDescriptor, AssetQuery, ListAssetsCache, ListAssetsError};

struct MockCacheEntry {
    descriptor: Vec<AssetDescriptor>,
    last_modified: SystemTime,
}

pub struct MockAssetCatalogCache {
    descriptors: RefCell<HashMap<AssetQuery, MockCacheEntry>>,
    max_age: Duration,
}

impl MockAssetCatalogCache {
    pub fn new(max_age: Duration) -> Self {
        MockAssetCatalogCache {
            descriptors: RefCell::new(HashMap::new()),
            max_age,
        }
    }
}

impl AssetCatalog for MockAssetCatalogCache {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        if let Some(entry) = self.descriptors.borrow().get(query) {
            if SystemTime::now()
                .duration_since(entry.last_modified)
                .unwrap_or(self.max_age)
                < self.max_age
            {
                return Ok(entry.descriptor.clone());
            }
        }
        Ok(vec![])
    }
}

impl ListAssetsCache for MockAssetCatalogCache {
    fn has_cache_entry(&self, query: &AssetQuery) -> bool {
        match self.descriptors.borrow().get(query) {
            Some(entry) => {
                SystemTime::now()
                    .duration_since(entry.last_modified)
                    .unwrap_or(self.max_age)
                    < self.max_age
            }
            None => false,
        }
    }
    fn save(&self, descriptor: &[AssetDescriptor], query: &AssetQuery) {
        let mut cache_map = self.descriptors.borrow_mut();
        cache_map.insert(
            query.clone(),
            MockCacheEntry {
                descriptor: descriptor.to_vec(),
                last_modified: SystemTime::now(),
            },
        );
    }
}
