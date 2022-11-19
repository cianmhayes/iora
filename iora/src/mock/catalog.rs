use std::cell::RefCell;

use crate::{AssetCatalog, AssetDescriptor, AssetQuery, ListAssetsError};

pub struct MockAssetCatalog {
    pub descriptors: RefCell<Vec<AssetDescriptor>>,
}

impl Default for MockAssetCatalog {
    fn default() -> Self {
        MockAssetCatalog {
            descriptors: RefCell::new(vec![]),
        }
    }
}

impl AssetCatalog for MockAssetCatalog {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        AssetDescriptor::filter_to_matching(&self.descriptors.borrow(), query)
    }
}
