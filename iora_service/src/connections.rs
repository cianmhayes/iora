use axum::async_trait;
use bb8::ManageConnection;
use thiserror::Error;

use iora::AzureBlobAssetIndex;

pub struct IoraServiceState {
    pub asset_index_connection_pool: bb8::Pool<AssetIndexConnectionManager>,
}

impl IoraServiceState {
    pub async fn new(
        asset_index_connection_type: AssetIndexConnectionType,
    ) -> Result<Self, AssetIndexConnectionError> {
        Ok(IoraServiceState {
            asset_index_connection_pool: bb8::Pool::builder()
                .build(AssetIndexConnectionManager {
                    asset_index_connection_type,
                })
                .await?,
        })
    }
}

pub enum AssetIndexConnectionType {
    AzureBlobAssetIndex {
        storage_account_name: String,
        blob_container_name: String,
        sas_token: String,
    },
}

pub struct AssetIndexConnectionManager {
    pub asset_index_connection_type: AssetIndexConnectionType,
}

#[derive(Error,Debug)]
pub enum AssetIndexConnectionError {}

#[async_trait]
impl ManageConnection for AssetIndexConnectionManager {
    type Connection = iora::AzureBlobAssetIndex;
    type Error = AssetIndexConnectionError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match &self.asset_index_connection_type {
            AssetIndexConnectionType::AzureBlobAssetIndex {
                storage_account_name,
                blob_container_name,
                sas_token,
            } => Ok(AzureBlobAssetIndex::new(
                storage_account_name,
                blob_container_name,
                sas_token,
            )),
        }
    }

    async fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
