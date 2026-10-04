/// Local embedding model files: resolve, verify, download
///
/// The model is pinned to one Hugging Face revision, with each file's size and
/// SHA-256 compiled in. Files are only handed to the loader after they verify.
/// Downloads stream into `<name>.part`, hashed as they arrive, and are renamed
/// into place only when the hash matches.
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use zero_latency_core::{Result, ZeroLatencyError};

/// Directory name under `~/.zero-latency/models/`
pub const BGE_MODEL_NAME: &str = "bge-small-en-v1.5";

/// Default download source: `<base>/resolve/<revision>/<file>`
pub const BGE_BASE_URL: &str = "https://huggingface.co/BAAI/bge-small-en-v1.5";

/// Pinned commit of BAAI/bge-small-en-v1.5 (2024-02-22).
///
/// Sizes and hashes come from `GET /api/models/BAAI/bge-small-en-v1.5/tree/<revision>`:
/// `lfs.oid` for `onnx/model.onnx`; `tokenizer.json` isn't in LFS, so its SHA-256
/// is from a download whose git blob id matched the listed `oid`. CI's model
/// cache is keyed on this line: keep it a plain string literal.
pub const BGE_REVISION: &str = "5c38ec7c405ec4b44b94cc5a9bb96e735b38267a";
const BGE_MODEL_ONNX_SIZE: u64 = 133_093_490;
const BGE_MODEL_ONNX_SHA256: &str =
    "828e1496d7fabb79cfa4dcd84fa38625c0d3d21da474a00f08db0f559940cf35";
const BGE_TOKENIZER_SIZE: u64 = 711_396;
const BGE_TOKENIZER_SHA256: &str =
    "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66";

/// One file the model needs
#[derive(Debug, Clone)]
pub struct ModelFile {
    /// Name inside the model directory, e.g. `model.onnx`
    pub name: String,
    /// Path inside the repository at the pinned revision, e.g. `onnx/model.onnx`
    pub remote_path: String,
    pub size: u64,
    /// Lowercase hex SHA-256
    pub sha256: String,
}

/// A pinned model: where it lives and what its files must hash to
#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub name: String,
    pub revision: String,
    pub base_url: String,
    pub files: Vec<ModelFile>,
}

impl ModelSpec {
    /// BAAI/bge-small-en-v1.5, ONNX export
    pub fn bge_small_en_v1_5() -> Self {
        Self {
            name: BGE_MODEL_NAME.to_string(),
            revision: BGE_REVISION.to_string(),
            base_url: BGE_BASE_URL.to_string(),
            files: vec![
                ModelFile {
                    name: "model.onnx".to_string(),
                    remote_path: "onnx/model.onnx".to_string(),
                    size: BGE_MODEL_ONNX_SIZE,
                    sha256: BGE_MODEL_ONNX_SHA256.to_string(),
                },
                ModelFile {
                    name: "tokenizer.json".to_string(),
                    remote_path: "tokenizer.json".to_string(),
                    size: BGE_TOKENIZER_SIZE,
                    sha256: BGE_TOKENIZER_SHA256.to_string(),
                },
            ],
        }
    }
}

/// Where the model's files are, and whether we may download into it
#[derive(Debug, Clone)]
pub struct ModelStore {
    spec: ModelSpec,
    dir: PathBuf,
    /// Set from `embedding.local_model_path`: read-only, never downloaded into
    user_supplied: bool,
}

impl ModelStore {
    /// Resolve the directory: `model_path` if set, otherwise
    /// `~/.zero-latency/models/<name>`. `base_url` overrides the download source.
    pub fn new(
        mut spec: ModelSpec,
        model_path: Option<PathBuf>,
        base_url: Option<String>,
    ) -> Result<Self> {
        if let Some(url) = base_url {
            spec.base_url = url.trim_end_matches('/').to_string();
        }
        let (dir, user_supplied) = match model_path {
            Some(path) => (path, true),
            None => {
                let home = dirs::home_dir().ok_or_else(|| {
                    ZeroLatencyError::configuration(
                        "Cannot find the home directory for the model cache; set ZL_EMBEDDING_LOCAL_MODEL_PATH",
                    )
                })?;
                (
                    home.join(".zero-latency").join("models").join(&spec.name),
                    false,
                )
            }
        };
        Ok(Self {
            spec,
            dir,
            user_supplied,
        })
    }

    /// Store for `doc-indexer --fetch-model [DIR]`. An explicit `target` is
    /// downloaded into; without one this is the same as `new`.
    pub fn for_fetch(
        spec: ModelSpec,
        target: Option<PathBuf>,
        model_path: Option<PathBuf>,
        base_url: Option<String>,
    ) -> Result<Self> {
        let mut store = Self::new(spec, target.clone().or(model_path), base_url)?;
        if target.is_some() {
            store.user_supplied = false;
        }
        Ok(store)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    /// Path of a model file by name, whether or not it exists yet
    pub fn path_of(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Make every file present and verified, downloading into the default
    /// directory when needed. Returns the model directory.
    pub async fn ensure(&self) -> Result<PathBuf> {
        if !self.user_supplied {
            tokio::fs::create_dir_all(&self.dir).await.map_err(|e| {
                ZeroLatencyError::io(format!(
                    "Cannot create model directory {}: {}",
                    self.dir.display(),
                    e
                ))
            })?;
        }

        for file in &self.spec.files {
            self.ensure_file(file).await?;
        }
        Ok(self.dir.clone())
    }

    async fn ensure_file(&self, file: &ModelFile) -> Result<()> {
        let path = self.path_of(&file.name);

        if !path.exists() {
            if self.user_supplied {
                return Err(ZeroLatencyError::configuration(format!(
                    "{} not found in {} (set by ZL_EMBEDDING_LOCAL_MODEL_PATH; files there are never downloaded). Run `doc-indexer --fetch-model` elsewhere and copy the directory, or unset the path to download into ~/.zero-latency/models",
                    file.name,
                    self.dir.display()
                )));
            }
            return self.download(file).await;
        }

        match verify_file(&path, file).await? {
            Verification::Ok => Ok(()),
            Verification::Mismatch(actual) if self.user_supplied => {
                Err(ZeroLatencyError::configuration(format!(
                    "{} failed verification: expected {}, got {}",
                    path.display(),
                    describe(file.size, &file.sha256),
                    actual
                )))
            }
            Verification::Mismatch(actual) => {
                tracing::warn!(
                    "{} failed verification (expected {}, got {}); downloading it again",
                    path.display(),
                    describe(file.size, &file.sha256),
                    actual
                );
                let _ = tokio::fs::remove_file(&path).await;
                self.download(file).await
            }
        }
    }

    fn url_of(&self, file: &ModelFile) -> String {
        format!(
            "{}/resolve/{}/{}",
            self.spec.base_url, self.spec.revision, file.remote_path
        )
    }

    async fn download(&self, file: &ModelFile) -> Result<()> {
        let url = self.url_of(file);
        let path = self.path_of(&file.name);
        let part = self.path_of(&format!("{}.part", file.name));

        tracing::info!(
            "Downloading {} ({:.1} MB) from {} to {}",
            file.name,
            file.size as f64 / 1_048_576.0,
            url,
            self.dir.display()
        );

        let result = self.download_to(&url, &part, file).await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(&part).await;
        }
        result?;

        tokio::fs::rename(&part, &path).await.map_err(|e| {
            ZeroLatencyError::io(format!("Cannot move {} into place: {}", part.display(), e))
        })?;
        tracing::info!("Verified {}", path.display());
        Ok(())
    }

    async fn download_to(&self, url: &str, part: &Path, file: &ModelFile) -> Result<()> {
        let unreachable = |e: &dyn std::fmt::Display| {
            ZeroLatencyError::network(format!(
                "Cannot download {} from {}: {}. Run `doc-indexer --fetch-model` on a machine with access and copy {} here, or point ZL_EMBEDDING_LOCAL_MODEL_PATH at a directory holding the files",
                file.name,
                url,
                e,
                self.dir.display()
            ))
        };

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| unreachable(&e))?;
        let mut response = client.get(url).send().await.map_err(|e| unreachable(&e))?;
        if !response.status().is_success() {
            return Err(unreachable(&format!("HTTP {}", response.status())));
        }

        let mut out = tokio::fs::File::create(part)
            .await
            .map_err(|e| ZeroLatencyError::io(format!("Cannot write {}: {}", part.display(), e)))?;
        let mut hasher = Sha256::new();
        let mut size = 0u64;
        while let Some(chunk) = response.chunk().await.map_err(|e| unreachable(&e))? {
            hasher.update(&chunk);
            size += chunk.len() as u64;
            out.write_all(&chunk).await.map_err(|e| {
                ZeroLatencyError::io(format!("Cannot write {}: {}", part.display(), e))
            })?;
        }
        out.flush()
            .await
            .map_err(|e| ZeroLatencyError::io(format!("Cannot write {}: {}", part.display(), e)))?;

        let sha256 = hex(&hasher.finalize());
        if size != file.size || sha256 != file.sha256 {
            return Err(ZeroLatencyError::validation(
                "model_file",
                format!(
                    "Downloaded {} from {} failed verification: expected {}, got {}",
                    file.name,
                    url,
                    describe(file.size, &file.sha256),
                    describe(size, &sha256)
                ),
            ));
        }
        Ok(())
    }
}

enum Verification {
    Ok,
    /// Description of what was found instead
    Mismatch(String),
}

/// Check size first (cheap), then SHA-256, on the blocking pool
async fn verify_file(path: &Path, file: &ModelFile) -> Result<Verification> {
    let path = path.to_path_buf();
    let file = file.clone();
    tokio::task::spawn_blocking(move || {
        let size = std::fs::metadata(&path)
            .map_err(|e| ZeroLatencyError::io(format!("Cannot read {}: {}", path.display(), e)))?
            .len();
        if size != file.size {
            return Ok(Verification::Mismatch(format!("{} bytes", size)));
        }
        let sha256 = sha256_file(&path)?;
        if sha256 != file.sha256 {
            return Ok(Verification::Mismatch(describe(size, &sha256)));
        }
        Ok(Verification::Ok)
    })
    .await
    .map_err(|e| ZeroLatencyError::internal(format!("Model verification task failed: {}", e)))?
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut reader = std::fs::File::open(path)
        .map_err(|e| ZeroLatencyError::io(format!("Cannot read {}: {}", path.display(), e)))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| ZeroLatencyError::io(format!("Cannot read {}: {}", path.display(), e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn describe(size: u64, sha256: &str) -> String {
    format!("{} bytes, sha256 {}", size, sha256)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::mock_http::MockServer;
    use axum::http::{Method, StatusCode};

    const MODEL: &[u8] = b"fake onnx model bytes";
    const TOKENIZER: &[u8] = b"{\"fake\": \"tokenizer\"}";

    fn sha(bytes: &[u8]) -> String {
        hex(&Sha256::digest(bytes))
    }

    fn spec(base_url: &str) -> ModelSpec {
        ModelSpec {
            name: "fake-model".to_string(),
            revision: "abc123".to_string(),
            base_url: base_url.to_string(),
            files: vec![
                ModelFile {
                    name: "model.onnx".to_string(),
                    remote_path: "onnx/model.onnx".to_string(),
                    size: MODEL.len() as u64,
                    sha256: sha(MODEL),
                },
                ModelFile {
                    name: "tokenizer.json".to_string(),
                    remote_path: "tokenizer.json".to_string(),
                    size: TOKENIZER.len() as u64,
                    sha256: sha(TOKENIZER),
                },
            ],
        }
    }

    /// Serves the fake files at `/resolve/abc123/...`; `model` is what the
    /// model URL returns on each request (index = requests already seen)
    async fn server(model: Vec<&'static [u8]>) -> MockServer {
        MockServer::start_raw(move |request, seen| match request.route() {
            "/resolve/abc123/onnx/model.onnx" => {
                (StatusCode::OK, model[seen.min(model.len() - 1)].to_vec())
            }
            "/resolve/abc123/tokenizer.json" => (StatusCode::OK, TOKENIZER.to_vec()),
            _ => (StatusCode::NOT_FOUND, Vec::new()),
        })
        .await
    }

    /// A store using `dir` as if it were the default (downloadable) directory
    fn default_store(server: &MockServer, dir: &Path) -> ModelStore {
        ModelStore {
            spec: spec(&server.base_url),
            dir: dir.to_path_buf(),
            user_supplied: false,
        }
    }

    fn model_requests(server: &MockServer) -> usize {
        server
            .requests_to(Method::GET, "/resolve/abc123/onnx/model.onnx")
            .len()
    }

    #[tokio::test]
    async fn first_run_downloads_and_verifies() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("fake-model");
        let store = default_store(&server, &dir);

        assert_eq!(store.ensure().await.unwrap(), dir);
        assert_eq!(std::fs::read(dir.join("model.onnx")).unwrap(), MODEL);
        assert_eq!(
            std::fs::read(dir.join("tokenizer.json")).unwrap(),
            TOKENIZER
        );
        assert!(!dir.join("model.onnx.part").exists());
        assert_eq!(model_requests(&server), 1);
    }

    #[tokio::test]
    async fn verified_files_need_no_network() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("model.onnx"), MODEL).unwrap();
        std::fs::write(temp.path().join("tokenizer.json"), TOKENIZER).unwrap();
        // Nothing listens here: any request would fail
        let store = ModelStore {
            spec: spec("http://127.0.0.1:9"),
            dir: temp.path().to_path_buf(),
            user_supplied: false,
        };

        store.ensure().await.unwrap();
    }

    #[tokio::test]
    async fn corrupt_download_is_discarded() {
        let server = server(vec![b"tampered bytes!!!!!!!"]).await;
        let temp = tempfile::tempdir().unwrap();
        let store = default_store(&server, temp.path());

        let error = store.ensure().await.unwrap_err().to_string();

        assert!(error.contains("model.onnx"), "{}", error);
        assert!(error.contains(&sha(MODEL)), "{}", error);
        assert!(error.contains(&sha(b"tampered bytes!!!!!!!")), "{}", error);
        assert!(!temp.path().join("model.onnx").exists());
        assert!(!temp.path().join("model.onnx.part").exists());
    }

    #[tokio::test]
    async fn corrupt_cached_file_is_downloaded_again() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("model.onnx"), b"corrupt").unwrap();
        std::fs::write(temp.path().join("tokenizer.json"), TOKENIZER).unwrap();
        let store = default_store(&server, temp.path());

        store.ensure().await.unwrap();

        assert_eq!(
            std::fs::read(temp.path().join("model.onnx")).unwrap(),
            MODEL
        );
        assert_eq!(model_requests(&server), 1);
    }

    #[tokio::test]
    async fn corrupt_cached_file_fails_if_redownload_is_corrupt() {
        let server = server(vec![b"still wrong"]).await;
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("model.onnx"), b"corrupt").unwrap();
        let store = default_store(&server, temp.path());

        assert!(store.ensure().await.is_err());
        assert_eq!(model_requests(&server), 1);
        assert!(!temp.path().join("model.onnx").exists());
    }

    #[tokio::test]
    async fn user_supplied_directory_is_never_downloaded_into() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("tokenizer.json"), TOKENIZER).unwrap();
        let store = ModelStore::new(
            spec(&server.base_url),
            Some(temp.path().to_path_buf()),
            None,
        )
        .unwrap();

        let error = store.ensure().await.unwrap_err().to_string();

        assert!(error.contains("model.onnx"), "{}", error);
        assert!(
            error.contains(&temp.path().display().to_string()),
            "{}",
            error
        );
        assert!(server.requests().is_empty());
        assert!(!temp.path().join("model.onnx").exists());
    }

    #[tokio::test]
    async fn user_supplied_corrupt_file_names_both_hashes() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        let corrupt = b"fake onnx model byteZ"; // same size, different content
        std::fs::write(temp.path().join("model.onnx"), corrupt).unwrap();
        std::fs::write(temp.path().join("tokenizer.json"), TOKENIZER).unwrap();
        let store = ModelStore::new(
            spec(&server.base_url),
            Some(temp.path().to_path_buf()),
            None,
        )
        .unwrap();

        let error = store.ensure().await.unwrap_err().to_string();

        assert!(error.contains(&sha(MODEL)), "{}", error);
        assert!(error.contains(&sha(corrupt)), "{}", error);
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn unreachable_source_names_url_and_remedies() {
        let temp = tempfile::tempdir().unwrap();
        let store = ModelStore {
            spec: spec("http://127.0.0.1:9"),
            dir: temp.path().to_path_buf(),
            user_supplied: false,
        };

        let error = store.ensure().await.unwrap_err().to_string();

        assert!(
            error.contains("http://127.0.0.1:9/resolve/abc123/onnx/model.onnx"),
            "{}",
            error
        );
        assert!(error.contains("--fetch-model"), "{}", error);
        assert!(error.contains("ZL_EMBEDDING_LOCAL_MODEL_PATH"), "{}", error);
    }

    #[tokio::test]
    async fn fetch_downloads_into_explicit_target() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("app/models/fake-model");
        let store = ModelStore::for_fetch(
            spec("https://unused.example"),
            Some(target.clone()),
            None,
            Some(server.base_url.clone()),
        )
        .unwrap();

        assert_eq!(store.ensure().await.unwrap(), target);
        assert_eq!(std::fs::read(target.join("model.onnx")).unwrap(), MODEL);
        assert_eq!(model_requests(&server), 1);
    }

    #[tokio::test]
    async fn fetch_without_target_respects_user_supplied_path() {
        let server = server(vec![MODEL]).await;
        let temp = tempfile::tempdir().unwrap();
        let store = ModelStore::for_fetch(
            spec(&server.base_url),
            None,
            Some(temp.path().to_path_buf()),
            None,
        )
        .unwrap();

        assert!(store.ensure().await.is_err());
        assert!(server.requests().is_empty());
    }

    #[test]
    fn base_url_override_is_used() {
        let store = ModelStore::new(
            spec("https://huggingface.co/x"),
            Some(PathBuf::from("/tmp/m")),
            Some("http://mirror.local/x/".to_string()),
        )
        .unwrap();
        assert_eq!(
            store.url_of(&store.spec.files[0]),
            "http://mirror.local/x/resolve/abc123/onnx/model.onnx"
        );
    }
}
