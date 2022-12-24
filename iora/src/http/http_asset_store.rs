use crate::{validate_hash, AssetLocator, AssetPayload, AssetStore, AssetStoreError};

pub struct HttpAsssetStore {}

impl AssetStore for HttpAsssetStore {
    fn supports_locator(&self, locator: &AssetLocator) -> bool {
        matches!(locator.url.scheme(), "http" | "https")
    }

    fn fetch_by_locator(
        &self,
        locator: &AssetLocator,
        expected_hash: &str,
    ) -> Result<AssetPayload, AssetStoreError> {
        match reqwest::blocking::get(locator.url.as_str()) {
            Ok(resp) => match resp.bytes() {
                Ok(bytes) => {
                    let bytes_vec = bytes.to_vec();
                    validate_hash(&bytes_vec, expected_hash)?;
                    Ok(AssetPayload::Bytes(bytes.to_vec()))
                }
                Err(bytes_error) => Err(AssetStoreError::AssetStoreInternalError(
                    bytes_error.to_string(),
                )),
            },
            Err(request_error) => Err(AssetStoreError::AssetStoreInternalError(
                request_error.to_string(),
            )),
        }
    }
}
