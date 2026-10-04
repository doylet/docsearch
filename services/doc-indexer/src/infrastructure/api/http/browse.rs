/// Directory browsing for the indexing path picker
///
/// Browsing is restricted to a configured set of root directories. Roots are
/// canonicalised once at startup and every requested path is canonicalised per
/// call (resolving `..` and symlinks) before it is compared against them.
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use zero_latency_core::{Result, ZeroLatencyError};

use super::handlers::{AppError, AppState};
use crate::application::content_processing::ContentTypeDetector;

/// Allowed browse roots and listing limits
#[derive(Debug, Clone, Default)]
pub struct BrowseConfig {
    /// Canonicalised root directories; empty disables browsing
    roots: Vec<PathBuf>,
    max_entries: usize,
}

impl BrowseConfig {
    /// Canonicalise the configured roots, dropping any that don't resolve to a directory.
    pub fn new(roots: &[PathBuf], max_entries: usize) -> Self {
        let roots = roots
            .iter()
            .filter_map(|root| match std::fs::canonicalize(root) {
                Ok(canonical) if canonical.is_dir() => Some(canonical),
                Ok(_) => {
                    tracing::warn!(root = %root.display(), "Ignoring browse root that is not a directory");
                    None
                }
                Err(e) => {
                    tracing::warn!(root = %root.display(), error = %e, "Ignoring browse root that cannot be resolved");
                    None
                }
            })
            .collect::<Vec<_>>();

        if roots.is_empty() {
            tracing::info!(
                "Directory browsing disabled; set DOC_INDEXER_BROWSE_ROOTS to enable it"
            );
        }

        Self { roots, max_entries }
    }

    fn root_containing(&self, path: &Path) -> Option<&Path> {
        self.roots
            .iter()
            .find(|root| path.starts_with(root))
            .map(PathBuf::as_path)
    }
}

/// Query parameters for browse endpoint
#[derive(Debug, Deserialize)]
pub struct BrowseQuery {
    pub path: Option<String>,
}

/// Response for browse endpoint
#[derive(Debug, Serialize)]
pub struct BrowseResponse {
    /// Canonical path of the listed directory; null when listing the roots
    pub path: Option<String>,
    /// Parent directory, or null at a root or when listing the roots
    pub parent_path: Option<String>,
    pub items: Vec<BrowseItem>,
    /// True when the listing was cut off at the configured maximum
    pub truncated: bool,
}

/// Directory item in browse response
#[derive(Debug, Serialize)]
pub struct BrowseItem {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
}

/// `GET /api/browse?path=<dir>`
pub async fn browse_directory(
    State(state): State<AppState>,
    Query(params): Query<BrowseQuery>,
) -> std::result::Result<Json<BrowseResponse>, AppError> {
    Ok(Json(browse(&state.browse, params.path.as_deref()).await?))
}

/// List a directory inside the allowed roots, or the roots themselves when `path` is `None`.
pub async fn browse(config: &BrowseConfig, path: Option<&str>) -> Result<BrowseResponse> {
    let Some(requested) = path.filter(|p| !p.is_empty()) else {
        return Ok(list_roots(config));
    };

    let canonical = resolve(config, Path::new(requested)).await?;
    let root = config
        .root_containing(&canonical)
        .ok_or_else(|| ZeroLatencyError::permission_denied("browse outside allowed roots"))?;

    let metadata = tokio::fs::metadata(&canonical)
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to read metadata: {}", e)))?;
    if !metadata.is_dir() {
        return Err(ZeroLatencyError::validation(
            "path",
            "Path is not a directory",
        ));
    }

    let parent_path = if canonical == root {
        None
    } else {
        canonical.parent().map(|p| p.to_string_lossy().to_string())
    };

    let (items, truncated) = list_directory(&canonical, config.max_entries).await?;

    Ok(BrowseResponse {
        path: Some(canonical.to_string_lossy().to_string()),
        parent_path,
        items,
        truncated,
    })
}

fn list_roots(config: &BrowseConfig) -> BrowseResponse {
    let items = config
        .roots
        .iter()
        .map(|root| BrowseItem {
            name: root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root.to_string_lossy().to_string()),
            path: root.to_string_lossy().to_string(),
            is_directory: true,
            size: None,
        })
        .collect();

    BrowseResponse {
        path: None,
        parent_path: None,
        items,
        truncated: false,
    }
}

/// Canonicalise a requested path.
///
/// A missing path is `404` only if it would lie inside a root; otherwise it is
/// `403`, so the endpoint doesn't reveal what exists outside the roots.
async fn resolve(config: &BrowseConfig, requested: &Path) -> Result<PathBuf> {
    match tokio::fs::canonicalize(requested).await {
        Ok(canonical) => Ok(canonical),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if config.root_containing(&normalise(requested)).is_some() {
                Err(ZeroLatencyError::not_found(requested.to_string_lossy()))
            } else {
                Err(ZeroLatencyError::permission_denied(
                    "browse outside allowed roots",
                ))
            }
        }
        Err(_) => Err(ZeroLatencyError::permission_denied(
            "browse outside allowed roots",
        )),
    }
}

/// Lexically resolve `.` and `..` without touching the filesystem.
fn normalise(path: &Path) -> PathBuf {
    let mut normalised = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalised.pop();
            }
            other => normalised.push(other),
        }
    }
    normalised
}

/// Non-hidden sub-directories and indexable files, directories first, each sorted by name.
async fn list_directory(dir: &Path, max_entries: usize) -> Result<(Vec<BrowseItem>, bool)> {
    let mut entries = tokio::fs::read_dir(dir)
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to read directory: {}", e)))?;

    let mut directories = Vec::new();
    let mut files = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| ZeroLatencyError::internal(format!("Failed to read directory entry: {}", e)))?
    {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        let entry_path = entry.path();
        let Ok(mut file_type) = entry.file_type().await else {
            continue;
        };
        // Report a symlink by its target's type; browsing into it is still root-checked
        if file_type.is_symlink() {
            match tokio::fs::metadata(&entry_path).await {
                Ok(metadata) => file_type = metadata.file_type(),
                Err(_) => continue,
            }
        }

        if file_type.is_dir() {
            directories.push(BrowseItem {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_directory: true,
                size: None,
            });
        } else if file_type.is_file() && is_indexable(&entry_path) {
            let size = tokio::fs::metadata(&entry_path).await.ok().map(|m| m.len());
            files.push(BrowseItem {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_directory: false,
                size,
            });
        }
    }

    directories.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));

    let mut items = directories;
    items.extend(files);
    let truncated = items.len() > max_entries;
    items.truncate(max_entries);

    Ok((items, truncated))
}

fn is_indexable(path: &Path) -> bool {
    ContentTypeDetector::detect_from_path(path).is_some_and(|t| t.should_index())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn names(response: &BrowseResponse) -> Vec<&str> {
        response.items.iter().map(|i| i.name.as_str()).collect()
    }

    fn setup() -> (tempfile::TempDir, BrowseConfig) {
        let root = tempfile::tempdir().unwrap();
        let config = BrowseConfig::new(&[root.path().to_path_buf()], 1000);
        (root, config)
    }

    fn path_str(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn lists_directories_then_indexable_files_without_hidden_entries() {
        let (root, config) = setup();
        let docs = root.path().join("docs");
        fs::create_dir_all(docs.join("guides")).unwrap();
        fs::create_dir_all(docs.join(".git")).unwrap();
        fs::write(docs.join("b.pdf"), b"%PDF").unwrap();
        fs::write(docs.join("a.md"), "# a").unwrap();
        fs::write(docs.join(".hidden.md"), "# hidden").unwrap();

        let response = browse(&config, Some(&path_str(&docs))).await.unwrap();

        assert_eq!(names(&response), vec!["guides", "a.md", "b.pdf"]);
        assert!(response.items[0].is_directory);
        assert!(!response.items[1].is_directory);
        assert!(!response.truncated);
        assert_eq!(
            response.parent_path,
            Some(path_str(&fs::canonicalize(root.path()).unwrap()))
        );
    }

    #[tokio::test]
    async fn hides_non_indexable_files() {
        let (root, config) = setup();
        fs::write(root.path().join("app.exe"), b"MZ").unwrap();
        fs::write(root.path().join("photo.jpg"), b"\xff\xd8").unwrap();
        fs::write(root.path().join("notes.md"), "# notes").unwrap();

        let response = browse(&config, Some(&path_str(root.path()))).await.unwrap();

        assert_eq!(names(&response), vec!["notes.md"]);
        assert_eq!(response.parent_path, None, "parent is null at a root");
    }

    #[tokio::test]
    async fn no_path_lists_the_roots() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let config = BrowseConfig::new(&[a.path().to_path_buf(), b.path().to_path_buf()], 1000);

        let response = browse(&config, None).await.unwrap();

        let paths: Vec<_> = response.items.iter().map(|i| i.path.clone()).collect();
        assert_eq!(
            paths,
            vec![
                path_str(&fs::canonicalize(a.path()).unwrap()),
                path_str(&fs::canonicalize(b.path()).unwrap()),
            ]
        );
        assert!(response.items.iter().all(|i| i.is_directory));
        assert_eq!(response.path, None);
        assert_eq!(response.parent_path, None);
    }

    #[tokio::test]
    async fn no_roots_configured_lists_nothing_and_refuses_paths() {
        let config = BrowseConfig::new(&[], 1000);

        assert!(browse(&config, None).await.unwrap().items.is_empty());
        assert!(matches!(
            browse(&config, Some("/")).await,
            Err(ZeroLatencyError::PermissionDenied { .. })
        ));
    }

    #[tokio::test]
    async fn path_traversal_is_forbidden() {
        let (root, config) = setup();
        let requested = format!("{}/../../etc", root.path().display());

        let result = browse(&config, Some(&requested)).await;

        assert!(matches!(
            result,
            Err(ZeroLatencyError::PermissionDenied { .. })
        ));
    }

    #[tokio::test]
    async fn absolute_path_outside_roots_is_forbidden() {
        let (_root, config) = setup();

        let result = browse(&config, Some("/")).await;

        assert!(matches!(
            result,
            Err(ZeroLatencyError::PermissionDenied { .. })
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn symlink_escaping_a_root_is_forbidden() {
        let (root, config) = setup();
        let outside = tempfile::tempdir().unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();

        let result = browse(&config, Some(&path_str(&link))).await;

        assert!(matches!(
            result,
            Err(ZeroLatencyError::PermissionDenied { .. })
        ));
    }

    #[tokio::test]
    async fn missing_directory_inside_root_is_not_found() {
        let (root, config) = setup();
        let missing = fs::canonicalize(root.path()).unwrap().join("missing");

        let result = browse(&config, Some(&path_str(&missing))).await;

        assert!(matches!(result, Err(ZeroLatencyError::NotFound { .. })));
    }

    #[tokio::test]
    async fn missing_path_outside_roots_is_forbidden() {
        let (_root, config) = setup();

        let result = browse(&config, Some("/definitely/not/here")).await;

        assert!(matches!(
            result,
            Err(ZeroLatencyError::PermissionDenied { .. })
        ));
    }

    #[tokio::test]
    async fn file_path_is_a_bad_request() {
        let (root, config) = setup();
        let file = root.path().join("a.md");
        fs::write(&file, "# a").unwrap();

        let result = browse(&config, Some(&path_str(&file))).await;

        assert!(matches!(result, Err(ZeroLatencyError::Validation { .. })));
    }

    #[tokio::test]
    async fn listing_is_truncated_at_the_cap() {
        let root = tempfile::tempdir().unwrap();
        for i in 0..5 {
            fs::write(root.path().join(format!("{i}.md")), "# doc").unwrap();
        }
        let config = BrowseConfig::new(&[root.path().to_path_buf()], 3);

        let response = browse(&config, Some(&path_str(root.path()))).await.unwrap();

        assert_eq!(response.items.len(), 3);
        assert!(response.truncated);
    }
}
