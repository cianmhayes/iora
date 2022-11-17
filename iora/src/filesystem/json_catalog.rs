use crate::{AssetCatalog, AssetDescriptor, AssetQuery, ListAssetsCacheError, ListAssetsError};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub struct JsonFileAssetCatalog {
    storage_path: PathBuf,
}

impl JsonFileAssetCatalog {
    pub fn new(file_path: &Path) -> Self {
        JsonFileAssetCatalog {
            storage_path: file_path.to_path_buf(),
        }
    }

    fn read_descriptors(&self) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        if let Ok(file) = File::open(&self.storage_path) {
            let reader = BufReader::new(file);
            if let Ok(descriptors) = serde_json::from_reader::<_, Vec<AssetDescriptor>>(reader) {
                return Ok(descriptors);
            }
        }
        return Err(ListAssetsError::QueryFailed);
    }

    pub fn populate_file(
        &self,
        descriptors: Vec<AssetDescriptor>,
    ) -> Result<(), ListAssetsCacheError> {
        if let Ok(file) = File::options()
            .create(true)
            .write(true)
            .open(&self.storage_path)
        {
            let writer = BufWriter::new(file);
            if let Ok(()) = serde_json::to_writer_pretty(writer, &descriptors) {
                return Ok(());
            }
        }
        return Err(ListAssetsCacheError::StorageError);
    }
}

impl AssetCatalog for JsonFileAssetCatalog {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        match self.read_descriptors() {
            Ok(existing_descriptors) => {
                AssetDescriptor::filter_to_matching(&existing_descriptors, query)
            }
            Err(e) => Err(e),
        }
    }
}
