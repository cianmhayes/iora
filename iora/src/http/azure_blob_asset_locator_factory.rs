use crate::AssetLocator;
use reqwest::Url;
use std::str::FromStr;

pub enum AzureBlobAssetLocatorFactoryError {
    FailedToConstructUrl(String),
}

pub trait AzureBlobAssetLocatorFactory {
    fn get_locator(
        &self,
        service_endpoint: &str,
        container_name: &str,
        blob_name: &str,
    ) -> Result<AssetLocator, AzureBlobAssetLocatorFactoryError>;
}

pub struct AzureBlobStorageDirectAccessLocatorFactory {
    pub sas_token: String,
}

impl AzureBlobAssetLocatorFactory for AzureBlobStorageDirectAccessLocatorFactory {
    fn get_locator(
        &self,
        service_endpoint: &str,
        container_name: &str,
        blob_name: &str,
    ) -> Result<AssetLocator, AzureBlobAssetLocatorFactoryError> {
        let url = format!(
            "{}{}/{}?{}",
            service_endpoint, container_name, blob_name, self.sas_token
        );
        match Url::from_str(&url) {
            Ok(u) => Ok(AssetLocator {
                locator_type: "AzureBlobStorageDirectAccess".to_owned(),
                url: u,
            }),
            Err(e) => Err(AzureBlobAssetLocatorFactoryError::FailedToConstructUrl(
                e.to_string(),
            )),
        }
    }
}
