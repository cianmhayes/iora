use crate::{AssetDescriptor, AssetIndex, AssetQuery, ListAssetsError};

#[derive(Debug)]
pub struct HttpAssetIndex {
    target_host: String,
}

impl HttpAssetIndex {
    pub fn new(target_host: &str) -> Self {
        HttpAssetIndex {
            target_host: if target_host.starts_with("http://")
                || target_host.starts_with("https://")
            {
                target_host.to_owned()
            } else {
                "https://".to_owned() + target_host
            },
        }
    }
}

impl AssetIndex for HttpAssetIndex {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        let url = format!(
            "{}/assets?name={}{}",
            self.target_host,
            query.name_constraint.to_string(),
            if let Some(vc) = &query.version_constraint {
                "&version=".to_owned() + &vc.to_string()
            } else {
                "".to_owned()
            }
        );

        match reqwest::blocking::get(url) {
            Ok(resp) => match resp.json::<Vec<AssetDescriptor>>() {
                Ok(result) => Ok(result),
                Err(json_error) => Err(ListAssetsError::AssetIndexInternalError(format!(
                    "Faield to parse response: {}",
                    json_error
                ))),
            },
            Err(request_error) => Err(ListAssetsError::AssetIndexInternalError(format!(
                "Service request failed: {}",
                request_error
            ))),
        }
    }
}
