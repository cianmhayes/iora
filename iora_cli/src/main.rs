use clap::{Parser, Subcommand};
use iora::{AssetQuery, ConstraintParsingError, ListAssetsError, AssetStoreError};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::{filter, prelude::*};

use thiserror::Error;

#[derive(Error, Debug)]
enum IoraCliError {
    #[error("Unsupported asset query parameters: {0}")]
    FindArgumentError(ConstraintParsingError),
    #[error("Error occurred while listing assets: {0}")]
    FindError(ListAssetsError),
    #[error("Error occurred while fetching asset: {0}")]
    FetchError(AssetStoreError),
    #[error("No matching asset available for downloading.")]
    FetchErrorNoMatchingAsset,
    #[error("Query parameters matched multiple assets.")]
    FetchErrorTooManyMatchingAssets
}

impl From<ConstraintParsingError> for IoraCliError {
    fn from(e: ConstraintParsingError) -> Self {
        IoraCliError::FindArgumentError(e)
    }
}

impl From<ListAssetsError> for IoraCliError {
    fn from(e: ListAssetsError) -> Self {
        IoraCliError::FindError(e)
    }
}

impl From<AssetStoreError> for IoraCliError {
    fn from(e: AssetStoreError) -> Self {
        IoraCliError::FetchError(e)
    }
}

#[derive(Parser, Debug)]
#[command(name = "iora")]
#[command(bin_name = "iora_cli")]
struct IoraCli {
    #[command(subcommand)]
    command: IoraCommands,

    #[arg(long)]
    log: bool,

    #[arg(long)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum IoraCommands {
    #[command(arg_required_else_help = true)]
    Find(Find),
    #[command(arg_required_else_help = true)]
    Fetch(Fetch),
}

#[derive(clap::Args, Debug)]
#[command(about = "Find available packages.")]
struct Find {
    /// A pattern describing the range of asset names of interest.
    #[arg(short, long, value_name = "NAME_CONSTRAINT", required = true)]
    name: String,
    /// A pattern describing the range of asset versions of interest.
    #[arg(short, long, value_name = "VERSION_CONSTRAINT", required = false)]
    version: Option<String>,
}

impl Find {
    fn run(&self, catalog: &impl iora::AssetIndex) -> Result<(), IoraCliError> {
        let query = AssetQuery::new_from_strings(&self.name, &self.version)?;
        let results = catalog.list_assets(&query)?;
        print_asset_descriptor_table(&results);
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
#[command(about = "Fetch the desired package.")]
struct Fetch {
    /// A pattern describing the range of asset names of interest.
    #[arg(short, long, value_name = "NAME_CONSTRAINT", required = true)]
    name: String,
    /// A pattern describing the range of asset versions of interest.
    #[arg(short, long, value_name = "VERSION_CONSTRAINT", required = false)]
    version: Option<String>,
}

impl Fetch {
    fn run(&self, catalog: &impl iora::AssetIndex, store: &impl iora::AssetStore) -> Result<(), IoraCliError> {
        let query = AssetQuery::new_from_strings(&self.name, &self.version)?;
        let results = catalog.list_assets(&query)?;
        if results.is_empty() {
            Err(IoraCliError::FetchErrorNoMatchingAsset)
        } else if results.len() > 1 {
            Err(IoraCliError::FetchErrorTooManyMatchingAssets)
        } else {
            store.fetch_by_descriptor(results.first().unwrap())?;
            Ok(())
        }
    }
}

fn print_asset_descriptor_table(descriptors: &Vec<iora::AssetDescriptor>) {
    println!("{0: <32} {1: <32} {2: <32}", "Name", "Version", "Hash");
    for ad in descriptors {
        println!(
            "{0: <32} {1: <32} {2: <32}",
            ad.name,
            ad.version.to_string(),
            ad.content_hash
        );
    }
}

fn main() {
    let args = IoraCli::parse();

    if args.log {
        let stdout_log = tracing_subscriber::fmt::layer().with_ansi(false).pretty();
        tracing_subscriber::registry()
            .with(stdout_log.with_filter(if args.verbose {
                filter::LevelFilter::INFO
            } else {
                filter::LevelFilter::WARN
            }))
            .init();
    }

    let mut cache_path = std::env::current_dir().unwrap();
    cache_path.push(PathBuf::from(".cache"));
    if !cache_path.as_path().exists() {
        if let Err(e) = fs::create_dir_all(cache_path.as_path()) {
            println!("Error: {}", e);
            return;
        }
    }
    let catalog = iora::JsonFileAssetIndexCache::new(
        &cache_path.join(PathBuf::from("descriptors.json")),
        Duration::from_nanos(1),
        iora::HttpAssetIndex::new("http://localhost:3000"),
    );
    let store = match iora::FilesystemAssetStoreCache::new(&cache_path, iora::HttpAsssetStore {}) {
        Ok(store) => store,
        Err(e) => {
            print!("Could not configure the asset store: {}", e);
            return;
        }
    };

    let command_result = match args.command {
        IoraCommands::Find(f) => f.run(&catalog),
        IoraCommands::Fetch(f) => f.run(&catalog, &store),
    };
    match command_result {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    };
}
