use std::cell::RefCell;

use crate::{AssetIndex, AssetDescriptor, AssetQuery, ListAssetsError};

pub struct MockAssetIndex {
    pub descriptors: RefCell<Vec<AssetDescriptor>>,
}

impl Default for MockAssetIndex {
    fn default() -> Self {
        MockAssetIndex {
            descriptors: RefCell::new(vec![]),
        }
    }
}

impl AssetIndex for MockAssetIndex {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        AssetDescriptor::filter_to_matching(&self.descriptors.borrow(), query)
    }
}
