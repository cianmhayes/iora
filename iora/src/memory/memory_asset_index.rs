use std::cell::RefCell;

use crate::{AssetDescriptor, AssetIndex, AssetQuery, ListAssetsError};

pub struct MemoryAssetIndex {
    pub descriptors: RefCell<Vec<AssetDescriptor>>,
}

impl MemoryAssetIndex {
    pub fn new(initial_descriptors: Vec<AssetDescriptor>) -> Self {
        MemoryAssetIndex {
            descriptors: RefCell::new(initial_descriptors),
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::{AssetDescriptor, AssetIndex, AssetQuery, MemoryAssetIndex, SemVer};

    #[test]
    pub fn list_assets() {
        match MemoryAssetIndex::default().list_assets(&AssetQuery::new(
            &crate::NameConstraint::Contains("a".to_owned()),
            &None,
        )) {
            Ok(descriptors) => assert_eq!(descriptors.len(), 0),
            Err(_) => panic!("Unexpected error"),
        };

        match MemoryAssetIndex::new(vec![
            AssetDescriptor::new("asset", &SemVer::new(1, 0, 0, None, None), "hash"),
            AssetDescriptor::new("asset", &SemVer::new(1, 1, 0, None, None), "hash"),
        ])
        .list_assets(&AssetQuery::new(
            &crate::NameConstraint::Contains("a".to_owned()),
            &None,
        )) {
            Ok(descriptors) => assert_eq!(descriptors.len(), 1),
            Err(_) => panic!("Unexpected error"),
        };
    }
}
