use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zero_latency_contracts::config::helpers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub server_url: String,
    pub collection_name: String,
    pub default_limit: u32,
    pub output_format: String,
    pub verbose: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        let standard_config = helpers::default_cli_config();
        Self {
            server_url: standard_config.server_url(),
            collection_name: standard_config.collection_name,
            default_limit: 10,
            output_format: "table".to_string(),
            verbose: false,
        }
    }
}

impl CliConfig {
    pub fn load() -> Result<Self> {
        let config_file = Self::config_file()?;
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_file, content)?;
        Ok(())
    }

    pub fn set_collection(&mut self, collection_name: String) -> Result<()> {
        self.collection_name = collection_name;
        self.save()
    }

    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("mdx");

        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }
}
