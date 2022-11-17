use std::cell::RefCell;

use crate::{AssetCatalog, AssetDescriptor, ListAssetsError, AssetQuery};

pub struct MockAssetCatalog {
    pub descriptors: RefCell<Vec<AssetDescriptor>>,
}

impl MockAssetCatalog {
    pub fn new() -> Self {
        MockAssetCatalog {
            descriptors: RefCell::new(vec![]),
        }
    }
}

impl AssetCatalog for MockAssetCatalog {
    fn list_assets(
        &self,
        query:&AssetQuery,
    ) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        AssetDescriptor::filter_to_matching(
            &self.descriptors.borrow(),
            query,
        )
    }
}
