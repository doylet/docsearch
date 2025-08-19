use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn, debug};

/// Model manager for downloading and caching embedding models
pub struct ModelManager {
    cache_dir: PathBuf,
}

/// Information about an embedding model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub onnx_url: String,
    pub tokenizer_url: String,
    pub config_url: String,
    pub expected_dimensions: usize,
    pub model_size_mb: usize,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_directory()?;
        Ok(Self { cache_dir })
    }

    /// Get the cache directory for models (~/.cache/zero-latency/models/)
    fn get_cache_directory() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let cache_dir = home.join(".cache").join("zero-latency").join("models");
        Ok(cache_dir)
    }

    /// Get model information for gte-small
    pub fn get_gte_small_info() -> ModelInfo {
        ModelInfo {
            name: "gte-small".to_string(),
            // Using Hugging Face model repository
            onnx_url: "https://huggingface.co/thenlper/gte-small/resolve/main/onnx/model.onnx".to_string(),
            tokenizer_url: "https://huggingface.co/thenlper/gte-small/resolve/main/tokenizer.json".to_string(),
            config_url: "https://huggingface.co/thenlper/gte-small/resolve/main/config.json".to_string(),
            expected_dimensions: 384,
            model_size_mb: 120,
        }
    }

    /// Check if a model is available locally
    pub async fn is_model_available(&self, model_info: &ModelInfo) -> Result<bool> {
        let model_dir = self.cache_dir.join(&model_info.name);
        let onnx_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let config_path = model_dir.join("config.json");

        let onnx_exists = onnx_path.exists();
        let tokenizer_exists = tokenizer_path.exists();
        let config_exists = config_path.exists();

        if onnx_exists && tokenizer_exists && config_exists {
            // Verify file integrity
            self.verify_model_files(&model_dir, model_info).await
        } else {
            Ok(false)
        }
    }

    /// Download and cache a model
    pub async fn download_model(&self, model_info: &ModelInfo) -> Result<()> {
        info!("Downloading model: {}", model_info.name);
        
        let model_dir = self.cache_dir.join(&model_info.name);
        
        // Create model directory
        fs::create_dir_all(&model_dir).await
            .context("Failed to create model cache directory")?;

        // Download ONNX model
        info!("Downloading ONNX model ({} MB)...", model_info.model_size_mb);
        let onnx_path = model_dir.join("model.onnx");
        self.download_file(&model_info.onnx_url, &onnx_path).await
            .context("Failed to download ONNX model")?;

        // Download tokenizer
        info!("Downloading tokenizer...");
        let tokenizer_path = model_dir.join("tokenizer.json");
        self.download_file(&model_info.tokenizer_url, &tokenizer_path).await
            .context("Failed to download tokenizer")?;

        // Download config
        info!("Downloading model config...");
        let config_path = model_dir.join("config.json");
        self.download_file(&model_info.config_url, &config_path).await
            .context("Failed to download model config")?;

        // Verify downloaded files
        self.verify_model_files(&model_dir, model_info).await
            .context("Model verification failed after download")?;

        info!("Model {} successfully downloaded and cached", model_info.name);
        Ok(())
    }

    /// Download a file from URL to local path
    async fn download_file(&self, url: &str, path: &Path) -> Result<()> {
        debug!("Downloading {} to {}", url, path.display());
        
        let response = reqwest::get(url).await
            .context("Failed to start download")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let content = response.bytes().await
            .context("Failed to read download content")?;
        
        fs::write(path, content).await
            .context("Failed to write downloaded file")?;

        Ok(())
    }

    /// Verify model files are complete and valid
    async fn verify_model_files(&self, model_dir: &Path, model_info: &ModelInfo) -> Result<bool> {
        let onnx_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let config_path = model_dir.join("config.json");

        // Check if all files exist
        if !onnx_path.exists() || !tokenizer_path.exists() || !config_path.exists() {
            return Ok(false);
        }

        // Check file sizes are reasonable
        let onnx_metadata = fs::metadata(&onnx_path).await?;
        let onnx_size_mb = onnx_metadata.len() / (1024 * 1024);
        
        if onnx_size_mb < 50 || onnx_size_mb > 200 {
            warn!("ONNX model size {} MB seems unexpected (expected ~{})", 
                  onnx_size_mb, model_info.model_size_mb);
            return Ok(false);
        }

        // Basic JSON validation for config files
        let tokenizer_content = fs::read_to_string(&tokenizer_path).await?;
        if !tokenizer_content.trim().starts_with('{') {
            warn!("Tokenizer file does not appear to be valid JSON");
            return Ok(false);
        }

        let config_content = fs::read_to_string(&config_path).await?;
        if !config_content.trim().starts_with('{') {
            warn!("Config file does not appear to be valid JSON");
            return Ok(false);
        }

        debug!("Model files verified successfully");
        Ok(true)
    }

    /// Get paths to model files
    pub fn get_model_paths(&self, model_info: &ModelInfo) -> ModelPaths {
        let model_dir = self.cache_dir.join(&model_info.name);
        ModelPaths {
            onnx_path: model_dir.join("model.onnx"),
            tokenizer_path: model_dir.join("tokenizer.json"),
            config_path: model_dir.join("config.json"),
        }
    }

    /// Ensure model is available, downloading if necessary
    pub async fn ensure_model_available(&self, model_info: &ModelInfo) -> Result<ModelPaths> {
        if !self.is_model_available(model_info).await? {
            info!("Model {} not found locally, downloading...", model_info.name);
            self.download_model(model_info).await?;
        } else {
            info!("Model {} found in cache", model_info.name);
        }

        Ok(self.get_model_paths(model_info))
    }
}

/// Paths to model files
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub onnx_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub config_path: PathBuf,
}
