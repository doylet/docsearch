use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub docs_directory: PathBuf,
    pub qdrant_url: String,
    pub collection_name: String,
    pub openai_api_key: Option<String>,
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.docs_directory.exists() {
            anyhow::bail!("Docs path does not exist: {}", self.docs_directory.display());
        }

        if !self.docs_directory.is_dir() {
            anyhow::bail!("Docs path is not a directory: {}", self.docs_directory.display());
        }

        Ok(())
    }
}
