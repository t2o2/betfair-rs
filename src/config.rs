use anyhow::Result;
use serde::Deserialize;
use std::fs;
use toml;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct BetfairConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub pem_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub betfair: BetfairConfig,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        info!("Config: {:?}", config);
        Ok(config)
    }
}
