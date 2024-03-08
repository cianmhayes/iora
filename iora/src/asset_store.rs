use crate::{AssetDescriptor, AssetLocator};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetStoreError {
    #[error("Descriptor has no supported locator.")]
    NoSupportedLocator,
    #[error("This store doesn't support the specified scheme {0}")]
    UnsupportedScheme(String),
    #[error("The asset's hash doesn't match the expected hash. Expected: {expected} Actual: {actual}")]
    AssetHashMismatch {expected:String, actual:String},
    #[error("Failed to retrieve asset. Details: {0}")]
    AssetStoreInternalError(String),
    #[error("The store was not configured properly. Details: {0}")]
    MisconfiguredStore(String)
}

pub enum AssetPayload {
    Bytes(Vec<u8>),
}

pub trait AssetStore {
    fn supports_locator(&self, locator: &AssetLocator) -> bool;

    fn fetch_by_locator(
        &self,
        locator: &AssetLocator,
        expected_hash: &str,
    ) -> Result<AssetPayload, AssetStoreError>;

    fn fetch_by_descriptor(
        &self,
        descriptor: &AssetDescriptor,
    ) -> Result<AssetPayload, AssetStoreError> {
        for locator in descriptor.locators.iter() {
            if self.supports_locator(locator) {
                return self.fetch_by_locator(locator, &descriptor.content_hash);
            }
        }
        Err(AssetStoreError::NoSupportedLocator)
    }
}

pub fn validate_hash(content: &Vec<u8>, expected_hash: &str) -> Result<(), AssetStoreError> {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash_hex = hex::encode(hasher.finalize());
    if expected_hash != hash_hex {
        Err(AssetStoreError::AssetHashMismatch { expected: expected_hash.to_string(), actual: hash_hex })
    } else {
        Ok(())
    }
}
