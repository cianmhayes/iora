use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Parser, Subcommand};
use iora::{AssetQuery, ConstraintParsingError, ListAssetsError};

#[derive(Parser, Debug)]
#[command(name = "iora")]
#[command(bin_name = "iora_cli")]
struct IoraCli {
    #[command(subcommand)]
    command: IoraCommands,
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
    #[arg(short, long, value_name = "VERSION_CONSTRAINT", required = true)]
    version: Option<String>,
}

enum IoraCliError {
    FindArgumentError(ConstraintParsingError),
    FindError(ListAssetsError),
}

impl Find {
    fn run(&self, catalog: &impl iora::AssetCatalog) -> Result<(), IoraCliError> {
        match AssetQuery::new_from_strings(&self.name, &self.version) {
            Ok(query) => match catalog.list_assets(&query) {
                Ok(results) => {
                    print_asset_descriptor_table(&results);
                    Ok(())
                }
                Err(e) => Err(IoraCliError::FindError(e)),
            },
            Err(e) => Err(IoraCliError::FindArgumentError(e)),
        }
    }
}

#[derive(clap::Args, Debug)]
#[command(about = "Fetch the desired package.")]
struct Fetch {}

fn make_asset_catalog(file_path: &Path) -> impl iora::AssetCatalog {
    let cache = Box::new(iora::JsonFileAssetCatalogCache::new(
        file_path,
        Duration::from_nanos(1),
    ));
    let remote = Box::new(iora::HttpAssetCatalog::new("http://localhost:3000"));
    iora::CachingAssetCatalog::new(cache, remote)
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
    let mut cache_path = std::env::current_dir().unwrap();
    cache_path.push(PathBuf::from(".cache"));
    cache_path.push(PathBuf::from("descriptors.json"));
    let catalog = make_asset_catalog(&cache_path);
    match args.command {
        IoraCommands::Find(f) => match f.run(&catalog) {
            Ok(()) => {}
            Err(IoraCliError::FindArgumentError(_)) => println!("Faield to parse the constraints."),
            Err(IoraCliError::FindError(_)) => println!("Failed to execute find."),
        },
        IoraCommands::Fetch(_) => println!("Fetch not implemented yet."),
    };
}
