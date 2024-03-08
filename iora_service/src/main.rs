mod connections;
mod list_assets;
mod settings;

use connections::{AssetIndexConnectionType, IoraServiceState};
use list_assets::list_assets;
use settings::{Settings, IoraServiceParameters};

use axum::{extract::Extension, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = IoraServiceParameters::parse();
    let settings = Settings::new(&args).unwrap();
    let state = Arc::new(
        IoraServiceState::new(
            AssetIndexConnectionType::AzureBlobAssetIndex {
                storage_account_name: settings.asset_index.storage_account_name,
                blob_container_name: settings.asset_index.blob_container_name,
                sas_token: settings.asset_index.blob_sas_token }).await.unwrap());
    let app = Router::new()
        .route("/assets", get(list_assets))
        .layer(Extension(state));
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.service.port));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
