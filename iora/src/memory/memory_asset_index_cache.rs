use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::{AssetIndex, AssetDescriptor, AssetQuery, ListAssetsCache, ListAssetsError};

struct MemoryCacheEntry {
    descriptor: Vec<AssetDescriptor>,
    last_modified: SystemTime,
}

pub struct MemoryAssetIndexCache {
    descriptors: RefCell<HashMap<AssetQuery, MemoryCacheEntry>>,
    max_age: Duration,
}

impl MemoryAssetIndexCache {
    pub fn new(max_age: Duration) -> Self {
        MemoryAssetIndexCache {
            descriptors: RefCell::new(HashMap::new()),
            max_age,
        }
    }
}

impl AssetIndex for MemoryAssetIndexCache {
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

impl ListAssetsCache for MemoryAssetIndexCache {
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
            MemoryCacheEntry {
                descriptor: descriptor.to_vec(),
                last_modified: SystemTime::now(),
            },
        );
    }
}
