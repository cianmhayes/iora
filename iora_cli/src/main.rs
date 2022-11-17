use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "iora")]
#[command(bin_name = "iora_cli")]
struct IoraCli {
    #[command(subcommand)]
    command:IoraCommands,
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
    /// Name constraint description.
    #[arg(short, long, value_name = "NAME_CONSTRAINT", required = true)]
    name: String,
}

#[derive(clap::Args, Debug)]
#[command(about = "Fetch the desired package.")]
struct Fetch {}

fn make_asset_catalog(file_path: &Path) -> Box<dyn iora::AssetCatalog> {
    let cache = Box::new(iora::JsonFileAssetCatalogCache::new(
        file_path,
        Duration::from_nanos(1),
    ));
    let remote = Box::new(iora::MockAssetCatalog::new());
    remote
        .descriptors
        .borrow_mut()
        .push(iora::AssetDescriptor::new(
            "asset.a",
            &iora::SemVer::from_str("1.0.0").unwrap(),
            "",
        ));
    remote
        .descriptors
        .borrow_mut()
        .push(iora::AssetDescriptor::new(
            "asset.a",
            &iora::SemVer::from_str("2.0.0").unwrap(),
            "",
        ));
    remote
        .descriptors
        .borrow_mut()
        .push(iora::AssetDescriptor::new(
            "asset.a",
            &iora::SemVer::from_str("2.0.0-beta").unwrap(),
            "",
        ));
    Box::new(iora::CachingAssetCatalog::new(cache, remote))
}

fn main() {
    let args = IoraCli::parse();
    let mut cache_path = std::env::current_dir().unwrap();
    cache_path.push(PathBuf::from(".cache"));
    cache_path.push(PathBuf::from("descriptors.json"));
    let catalog = make_asset_catalog(&cache_path);
    match args.command {
        IoraCommands::Find(f) => {
            match catalog.list_assets(&iora::NameConstraint::Contains(f.name).into()) {
                Ok(results) => {
                    println!(
                        "{0: <32} {1: <32} {2: <32}",
                        "Name", "Version", "Hash"
                    );
                    for ad in results {
                        println!(
                            "{0: <32} {1: <32} {2: <32}",
                            ad.name, ad.version, ad.content_hash
                        );
                    }
                },
                Err(e) => {}
            }
        },
        _ => panic!("Not implemented")
    }
}
