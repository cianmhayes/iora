use crate::{AssetDescriptor, AssetQuery};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListAssetsError {
    #[error("Asset index is missing or unavailable.")]
    AssetIndexNotFound(Option<String>),
    #[error("Asset index refused access.")]
    AssetIndexAccessDenied(Option<String>),
    #[error("Failed to execute the query. Details: {0}")]
    AssetIndexInternalError(String),
    #[error("Failed to execute the query. Details: {details:?}. Query: {query:?}")]
    BadQuery { query: String, details: String },
}

pub trait AssetIndex {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError>;
}
