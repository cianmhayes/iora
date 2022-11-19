use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tracing::{event, instrument, Level};

use crate::{AssetIndex, AssetDescriptor, AssetQuery, ListAssetsCache, ListAssetsError};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CacheEntry {
    descriptor: Vec<AssetDescriptor>,
    query: AssetQuery,
    last_modified: SystemTime,
}

#[derive(Debug)]
pub struct JsonFileAssetIndexCache {
    storage_path: PathBuf,
    max_age: Duration,
}

impl JsonFileAssetIndexCache {
    #[instrument]
    pub fn new(file_path: &Path, max_age: Duration) -> Self {
        event!(Level::INFO, file_path_exists= file_path.exists());
        JsonFileAssetIndexCache {
            storage_path: file_path.to_path_buf(),
            max_age,
        }
    }
    fn read_from_file(&self) -> HashMap<u64, CacheEntry> {
        if let Ok(file) = File::open(&self.storage_path) {
            let reader = BufReader::new(file);
            if let Ok(descriptors) =
                serde_json::from_reader::<_, HashMap<u64, CacheEntry>>(reader)
            {
                return descriptors;
            }
        }
        HashMap::new()
    }

    #[instrument]
    fn save_to_file(&self, descriptors: HashMap<u64, CacheEntry>) {
        match OpenOptions::new().write(true).create(true).open(&self.storage_path)
        {
            Ok(file) => {
                let writer = BufWriter::new(file);
                if let Err(e) = serde_json::to_writer_pretty(writer, &descriptors) {
                    event!(Level::ERROR, error=e.to_string());
                }
            }
            Err(e) => event!(Level::ERROR, error=e.to_string()),
        };
    }

    fn cache_key(query:&AssetQuery) -> u64 {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }
}

impl AssetIndex for JsonFileAssetIndexCache {
    fn list_assets(&self, query: &AssetQuery) -> Result<Vec<AssetDescriptor>, ListAssetsError> {        
        if let Some(entry) = self.read_from_file().get(&Self::cache_key(query)) {
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

impl ListAssetsCache for JsonFileAssetIndexCache {
    fn has_cache_entry(&self, query: &AssetQuery) -> bool {
        
        match self.read_from_file().get(&Self::cache_key(query)) {
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
            Self::cache_key(query),
            CacheEntry {
                descriptor: descriptor.to_vec(),
                query: query.clone(),
                last_modified: SystemTime::now(),
            },
        );
        self.save_to_file(cache_map);
    }
}
