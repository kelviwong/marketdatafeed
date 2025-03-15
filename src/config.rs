use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub binance: BinanceConfig,
    pub okx: OkxConfig,
}

#[derive(Debug, Deserialize)]
pub struct BinanceConfig {
    pub base_url: String,
    pub symbol: String,
    pub enable: bool,
}

#[derive(Debug, Deserialize)]
pub struct OkxConfig {
    pub base_url: String,
    pub symbol: String,
    pub enable: bool,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}