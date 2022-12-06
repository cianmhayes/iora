use iora::{
    AssetIndex, AssetDescriptor, AssetQuery, CachingAssetIndex, MemoryAssetIndex,
    MemoryAssetIndexCache, SemVer,
};
use std::str::FromStr;
use std::time::Duration;

#[test]
fn list() {
    let cache = Box::new(MemoryAssetIndexCache::new(Duration::from_secs(1)));
    let remote = Box::new(MemoryAssetIndex::default());
    remote.descriptors.borrow_mut().push(AssetDescriptor::new(
        "asset_name",
        &SemVer::from_str("2.45.6").unwrap(),
        "asset_hash",
    ));
    remote.descriptors.borrow_mut().push(AssetDescriptor::new(
        "asset_name",
        &SemVer::from_str("3.45.6").unwrap(),
        "asset_hash",
    ));
    remote.descriptors.borrow_mut().push(AssetDescriptor::new(
        "asset_name",
        &SemVer::from_str("3.45.7").unwrap(),
        "asset_hash",
    ));
    let catalog = CachingAssetIndex::new(cache, remote);

    let result = catalog
        .list_assets(&AssetQuery::from((
            &iora::NameConstraint::Contains("asset".to_string()),
            &None,
        )))
        .expect("t");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].version, SemVer::from_str("3.45.7").unwrap());
}
