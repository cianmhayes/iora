use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "iora")]
#[command(bin_name = "iora_service")]
pub struct IoraServiceParameters {
    #[arg(short, long, value_name = "PORT", required = false)]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub storage_account_name: String,
    pub blob_container_name: String,
    pub blob_sas_token: String
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub asset_index: AssetIndex,
    pub service: Service
}

impl Settings {
    pub fn new(commandline:&IoraServiceParameters) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/test").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("iora"))
            .set_override("service.port", commandline.port)?
            .build()?;
        s.try_deserialize()
    }
}