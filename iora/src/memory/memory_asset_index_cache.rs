use crate::{AssetDescriptor, AssetIndex, AssetQuery, ListAssetsError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

struct MemoryCacheEntry {
    descriptor: Vec<AssetDescriptor>,
    last_modified: SystemTime,
}

pub struct MemoryAssetIndexCache<TInnerIndex>
where
    TInnerIndex: AssetIndex,
{
    descriptors: RefCell<HashMap<AssetQuery, MemoryCacheEntry>>,
    max_age: Duration,
    inner_index: TInnerIndex,
}

impl<TInnerIndex> MemoryAssetIndexCache<TInnerIndex>
where
    TInnerIndex: AssetIndex,
{
    pub fn new(max_age: Duration, inner_index: TInnerIndex) -> Self {
        MemoryAssetIndexCache {
            descriptors: RefCell::new(HashMap::new()),
            max_age,
            inner_index,
        }
    }
}

impl<TInnerIndex> AssetIndex for MemoryAssetIndexCache<TInnerIndex>
where
    TInnerIndex: AssetIndex,
{
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

        match self.inner_index.list_assets(query) {
            Ok(results) => {
                if let Ok(mut borrowed_descriptors) = self.descriptors.try_borrow_mut() {
                    borrowed_descriptors.insert(
                        query.clone(),
                        MemoryCacheEntry {
                            descriptor: results.clone(),
                            last_modified: SystemTime::now(),
                        },
                    );
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }
}
