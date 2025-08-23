use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        Self {
            server_url: "http://localhost:8081".to_string(),
            collection_name: "zero_latency_docs".to_string(),
            default_limit: 10,
            output_format: "table".to_string(),
            verbose: false,
        }
    }
}

impl CliConfig {
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        // For now, just return default config
        // In the future, we can load from ~/.config/mdx/config.toml
        Ok(Self::default())
    }
    
    #[allow(dead_code)]
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("mdx");
        
        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }
    
    #[allow(dead_code)]
    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }
}
