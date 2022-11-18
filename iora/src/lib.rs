mod asset_catalog;
mod asset_descriptor;
mod constraints;
mod filesystem;
mod http;
mod mock;

mod semver;

pub use asset_catalog::{
    AssetCatalog, CachingAssetCatalog, ListAssetsError, ListAssetsCache,
    ListAssetsCacheError
};
pub use asset_descriptor::AssetDescriptor;
pub use constraints::{NameConstraint, VersionConstraint, AssetQuery, ConstraintParsingError};
pub use filesystem::{JsonFileAssetCatalog, JsonFileAssetCatalogCache};
pub use http::HttpAssetCatalog;
pub use mock::{MockAssetCatalog, MockAssetCatalogCache};
pub use semver::{SemVer, SemVerParseEror};

mod algo;
mod regexes;