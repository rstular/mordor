use color_eyre::{eyre::bail, Result};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

use crate::utils::defaults;

pub mod database;
pub mod http;
pub mod modules;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: database::Settings,
    pub http: http::Settings,
    #[serde(with = "hex")]
    pub secret_key: Vec<u8>,
    #[serde(default = "defaults::store_access_entries")]
    pub store_access_entries: bool,
    pub modules: modules::Settings,
}

impl Settings {
    pub fn load(file_path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name(file_path))
            .add_source(Environment::default().keep_prefix(false).prefix("MORDOR"))
            .build()?
            .try_deserialize()
    }

    pub fn sanity_check(&self) -> Result<()> {
        if self.secret_key.len() < 32 {
            bail!("Secret key must be at least 32 bytes long");
        }

        Ok(())
    }
}
