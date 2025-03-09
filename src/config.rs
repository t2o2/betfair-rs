use std::fs;
use serde::Deserialize;
use anyhow::Result;
use toml;
use tracing::info;

#[derive(Debug)]
#[allow(dead_code)]
pub struct BetfairCredentials {
    pub username: String,
    pub password: String,
    pub p12: Vec<u8>,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct BetfairConfig {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub pfx_path: String,
    pub pfx_password: String,
}

#[derive(Debug, Deserialize)]
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