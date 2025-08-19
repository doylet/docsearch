use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

pub struct DocumentWatcher {
    docs_path: PathBuf,
    tx: mpsc::UnboundedSender<FileEvent>,
    _watcher: RecommendedWatcher, // Keep alive to prevent dropping
}

impl DocumentWatcher {
    pub fn new(docs_path: PathBuf) -> Result<(Self, mpsc::UnboundedReceiver<FileEvent>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let event_tx = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if let Err(e) = handle_file_event(event, &event_tx) {
                            error!("Error handling file event: {}", e);
                        }
                    }
                    Err(e) => error!("File watcher error: {}", e),
                }
            },
            Config::default(),
        )?;

        // Start watching the docs directory
        watcher.watch(&docs_path, RecursiveMode::Recursive)?;
        info!("Started watching directory: {}", docs_path.display());

        let document_watcher = Self {
            docs_path,
            tx,
            _watcher: watcher,
        };

        Ok((document_watcher, rx))
    }

    pub async fn stop(self) {
        info!("Stopping document watcher for: {}", self.docs_path.display());
        // Watcher will be dropped automatically
    }
}

fn handle_file_event(event: Event, tx: &mpsc::UnboundedSender<FileEvent>) -> Result<()> {
    // Only process markdown files
    let paths: Vec<_> = event.paths.into_iter()
        .filter(|path| is_markdown_file(path))
        .collect();

    if paths.is_empty() {
        return Ok(());
    }

    match event.kind {
        EventKind::Create(_) => {
            for path in paths {
                debug!("File created: {}", path.display());
                if let Err(e) = tx.send(FileEvent::Created(path.clone())) {
                    error!("Failed to send Created event for {}: {}", path.display(), e);
                }
            }
        }
        EventKind::Modify(_) => {
            for path in paths {
                debug!("File modified: {}", path.display());
                if let Err(e) = tx.send(FileEvent::Modified(path.clone())) {
                    error!("Failed to send Modified event for {}: {}", path.display(), e);
                }
            }
        }
        EventKind::Remove(_) => {
            for path in paths {
                debug!("File deleted: {}", path.display());
                if let Err(e) = tx.send(FileEvent::Deleted(path.clone())) {
                    error!("Failed to send Deleted event for {}: {}", path.display(), e);
                }
            }
        }
        EventKind::Access(_) => {
            // Ignore access events (reading files)
        }
        EventKind::Any | EventKind::Other => {
            warn!("Unknown file event type: {:?}", event.kind);
        }
    }

    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_file_watcher() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let docs_path = temp_dir.path().to_path_buf();

        let (watcher, mut rx) = DocumentWatcher::new(docs_path.clone())?;

        // Create a markdown file
        let test_file = docs_path.join("test.md");
        fs::write(&test_file, "# Test Document")?;

        // Wait for the event
        if let Some(event) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await? {
            match event {
                FileEvent::Created(path) => {
                    assert_eq!(path, test_file);
                }
                _ => panic!("Expected Created event"),
            }
        } else {
            panic!("No event received within timeout");
        }

        // Modify the file
        fs::write(&test_file, "# Modified Test Document")?;

        // Wait for the modify event
        if let Some(event) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await? {
            match event {
                FileEvent::Modified(path) => {
                    assert_eq!(path, test_file);
                }
                _ => panic!("Expected Modified event"),
            }
        } else {
            panic!("No modify event received within timeout");
        }

        // Delete the file
        fs::remove_file(&test_file)?;

        // Wait for the delete event
        if let Some(event) = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await? {
            match event {
                FileEvent::Deleted(path) => {
                    assert_eq!(path, test_file);
                }
                _ => panic!("Expected Deleted event"),
            }
        } else {
            panic!("No delete event received within timeout");
        }

        watcher.stop().await;
        Ok(())
    }

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.MD")));
        assert!(is_markdown_file(Path::new("/path/to/file.md")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("test")));
        assert!(!is_markdown_file(Path::new("test.mdx")));
    }
}
