use crate::{
    validate_hash, AssetDescriptor, AssetLocator, AssetPayload, AssetStore, AssetStoreError,
};
use reqwest::Url;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub struct FilesystemAssetStoreCache<TInnerStore>
where
    TInnerStore: AssetStore,
{
    storage_path: PathBuf,
    inner_store: TInnerStore,
}

impl<TInnerStore> FilesystemAssetStoreCache<TInnerStore>
where
    TInnerStore: AssetStore,
{
    pub fn new(storage_path: &Path, inner_store: TInnerStore) -> Result<Self, AssetStoreError> {
        if !storage_path.is_absolute() {
            Err(AssetStoreError::MisconfiguredStore(
                "Storage root must be an absolute path.".to_owned(),
            ))
        } else {
            Ok(FilesystemAssetStoreCache {
                storage_path: storage_path.to_owned(),
                inner_store,
            })
        }
    }

    fn get_local_path_for_descriptor(&self, descriptor: &AssetDescriptor) -> PathBuf {
        self.storage_path
            .join(&descriptor.name)
            .join(descriptor.version.to_string())
            .join("asset")
    }

    pub fn save_asset(
        &self,
        descriptor: &AssetDescriptor,
        payload: &AssetPayload,
    ) -> Option<AssetLocator> {
        let file_path = self.get_local_path_for_descriptor(descriptor);
        if let Some(folder) = file_path.parent() {
            if !folder.exists() && create_dir_all(folder).is_err() {
                return None;
            }
        }

        if !file_path.is_absolute() {
            return None;
        }

        let new_url = Url::from_file_path(&file_path);
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(file_path);

        if let (Ok(new_url), Ok(mut file)) = (new_url, file) {
            match payload {
                AssetPayload::Bytes(buf) => {
                    if let Ok(()) = file.write_all(buf) {
                        return Some(AssetLocator { url: new_url });
                    }
                }
            }
        }

        None
    }
}

impl<TInnerStore> AssetStore for FilesystemAssetStoreCache<TInnerStore>
where
    TInnerStore: AssetStore,
{
    fn supports_locator(&self, locator: &AssetLocator) -> bool {
        locator.url.scheme() == "file" || self.inner_store.supports_locator(locator)
    }

    fn fetch_by_locator(
        &self,
        locator: &AssetLocator,
        expected_hash: &str,
    ) -> Result<AssetPayload, AssetStoreError> {
        if locator.url.scheme() == "file" {
            if let Ok(mut f) = File::open(locator.url.as_str()) {
                let mut buf = vec![];
                match f.read_to_end(&mut buf) {
                    Ok(_) => {
                        validate_hash(&buf, expected_hash)?;
                        return Ok(AssetPayload::Bytes(buf));
                    }
                    Err(e) => return Err(AssetStoreError::AssetStoreInternalError(e.to_string())),
                }
            }
        }

        Err(crate::AssetStoreError::UnsupportedScheme(
            locator.url.scheme().to_owned(),
        ))
    }

    fn fetch_by_descriptor(
        &self,
        descriptor: &AssetDescriptor,
    ) -> Result<AssetPayload, AssetStoreError> {
        let asset_path = self.get_local_path_for_descriptor(descriptor);
        if asset_path.exists() {
            if let Ok(mut f) = File::open(asset_path) {
                let mut buffer = vec![];
                if f.read_to_end(&mut buffer).is_ok() {
                    return Ok(AssetPayload::Bytes(buffer));
                }
            }
        }

        let mut error = AssetStoreError::NoSupportedLocator;
        for locator in descriptor.locators.iter() {
            if locator.url.scheme() == "file" {
                match self.fetch_by_locator(locator, &descriptor.content_hash) {
                    Ok(payload) => return Ok(payload),
                    Err(e) => error = e,
                }
            } else if self.inner_store.supports_locator(locator) {
                match self
                    .inner_store
                    .fetch_by_locator(locator, &descriptor.content_hash)
                {
                    Ok(payload) => {
                        self.save_asset(descriptor, &payload);
                        return Ok(payload);
                    }
                    Err(e) => error = e,
                }
            }
        }
        Err(error)
    }
}
