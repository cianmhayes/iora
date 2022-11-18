use crate::{AssetCatalog, AssetDescriptor, AssetQuery, ListAssetsError};

pub struct HttpAssetCatalog {
    target_host: String,
}

impl HttpAssetCatalog {
    pub fn new(target_host: &str) -> Self {
        HttpAssetCatalog {
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

impl AssetCatalog for HttpAssetCatalog {
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
                Err(_) => Err(ListAssetsError::QueryFailed),
            },
            Err(e) => {print!("{:?}", e); Err(ListAssetsError::QueryFailed)},
        }
    }
}
