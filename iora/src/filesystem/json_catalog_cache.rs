use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::{AssetCatalog, AssetDescriptor, AssetQuery, ListAssetsCache, ListAssetsError};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CacheEntry {
    descriptor: Vec<AssetDescriptor>,
    last_modified: SystemTime,
}

pub struct JsonFileAssetCatalogCache {
    storage_path: PathBuf,
    max_age: Duration,
}

impl JsonFileAssetCatalogCache {
    pub fn new(file_path: &Path, max_age: Duration) -> Self {
        JsonFileAssetCatalogCache {
            storage_path: file_path.to_path_buf(),
            max_age,
        }
    }
    fn read_from_file(&self) -> HashMap<AssetQuery, CacheEntry> {
        if let Ok(file) = File::open(&self.storage_path) {
            let reader = BufReader::new(file);
            if let Ok(descriptors) =
                serde_json::from_reader::<_, HashMap<AssetQuery, CacheEntry>>(reader)
            {
                return descriptors;
            }
        }
        HashMap::new()
    }
    fn save_to_file(&self, descriptors: HashMap<AssetQuery, CacheEntry>) {
        if let Ok(file) = File::options()
            .create(true)
            .write(true)
            .open(&self.storage_path)
        {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer_pretty(writer, &descriptors);
        }
    }
}

impl AssetCatalog for JsonFileAssetCatalogCache {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {
        if let Some(entry) = self.read_from_file().get(query) {
            if SystemTime::now()
                .duration_since(entry.last_modified)
                .unwrap_or(self.max_age)
                < self.max_age
            {
                return Ok(entry.descriptor.to_vec());
            }
        }
        Ok(vec![])
    }
}

impl ListAssetsCache for JsonFileAssetCatalogCache {
    fn has_cache_entry(&self, query: &AssetQuery) -> bool {
        match self.read_from_file().get(query) {
            Some(entry) => {
                SystemTime::now()
                    .duration_since(entry.last_modified)
                    .unwrap_or(self.max_age)
                    < self.max_age
            }
            None => false,
        }
    }

    fn save(&self, descriptor: &[AssetDescriptor], query: &AssetQuery) {
        let mut cache_map = self.read_from_file();
        cache_map.insert(
            query.clone(),
            CacheEntry {
                descriptor: descriptor.to_vec(),
                last_modified: SystemTime::now(),
            },
        );
        self.save_to_file(cache_map);
    }
}
