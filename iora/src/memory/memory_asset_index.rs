use std::cell::RefCell;

use crate::{AssetIndex, AssetDescriptor, AssetQuery, ListAssetsError};

pub struct MemoryAssetIndex {
    pub descriptors: RefCell<Vec<AssetDescriptor>>,
}

impl Default for MemoryAssetIndex {
    fn default() -> Self {
        MemoryAssetIndex {
            descriptors: RefCell::new(vec![]),
        }
    }
}

impl AssetIndex for MemoryAssetIndex {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        AssetDescriptor::filter_to_matching(&self.descriptors.borrow(), query)
    }
}
