use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

#[derive(Debug, Clone)]
struct PendingEvent {
    event: FileEvent,
    last_seen: Instant,
}

pub struct DocumentWatcher {
    docs_path: PathBuf,
    _watcher: RecommendedWatcher,
    debounce_duration: Duration,
}

const DEFAULT_DEBOUNCE_MS: u64 = 300;

impl DocumentWatcher {
    pub fn new(docs_path: PathBuf) -> Result<(Self, mpsc::UnboundedReceiver<FileEvent>)> {
        Self::new_with_debounce(docs_path, Duration::from_millis(DEFAULT_DEBOUNCE_MS))
    }

    pub fn new_with_debounce(
        docs_path: PathBuf, 
        debounce_duration: Duration
    ) -> Result<(Self, mpsc::UnboundedReceiver<FileEvent>)> {
        let (raw_tx, raw_rx) = mpsc::unbounded_channel();
        let (coalesced_tx, coalesced_rx) = mpsc::unbounded_channel();

        // Start the event coalescing task
        tokio::spawn(Self::coalesce_events(raw_rx, coalesced_tx, debounce_duration));

        let event_tx = raw_tx.clone();
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
            _watcher: watcher,
            debounce_duration,
        };

        Ok((document_watcher, coalesced_rx))
    }

    /// Coalesce rapid file system events to avoid thrashing
    async fn coalesce_events(
        mut raw_rx: mpsc::UnboundedReceiver<FileEvent>,
        coalesced_tx: mpsc::UnboundedSender<FileEvent>,
        debounce_duration: Duration,
    ) {
        let mut pending_events: HashMap<PathBuf, PendingEvent> = HashMap::new();
        let mut cleanup_interval = interval(debounce_duration);

        loop {
            tokio::select! {
                // Handle incoming events
                Some(event) = raw_rx.recv() => {
                    let path = match &event {
                        FileEvent::Created(p) | FileEvent::Modified(p) | FileEvent::Deleted(p) => p.clone(),
                    };

                    pending_events.insert(path, PendingEvent {
                        event,
                        last_seen: Instant::now(),
                    });
                }

                // Periodically flush expired events
                _ = cleanup_interval.tick() => {
                    let now = Instant::now();
                    let mut to_send = Vec::new();
                    let mut to_remove = Vec::new();

                    for (path, pending) in &pending_events {
                        if now.duration_since(pending.last_seen) >= debounce_duration {
                            to_send.push(pending.event.clone());
                            to_remove.push(path.clone());
                        }
                    }

                    // Remove expired events
                    for path in to_remove {
                        pending_events.remove(&path);
                    }

                    // Send coalesced events
                    for event in to_send {
                        if let Err(e) = coalesced_tx.send(event) {
                            error!("Failed to send coalesced event: {}", e);
                            break;
                        }
                    }
                }
            }
        }
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
    async fn test_event_coalescing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let docs_path = temp_dir.path().to_path_buf();

        let (watcher, mut rx) = DocumentWatcher::new_with_debounce(
            docs_path.clone(),
            Duration::from_millis(100)
        )?;

        // Create a markdown file
        let test_file = docs_path.join("test.md");
        fs::write(&test_file, "# Test Document")?;

        // Quickly modify it multiple times
        for i in 0..5 {
            fs::write(&test_file, format!("# Test Document {}", i))?;
            sleep(Duration::from_millis(10)).await;
        }

        // Wait for debounce period
        sleep(Duration::from_millis(200)).await;

        // Should receive only one coalesced event
        let mut event_count = 0;
        while let Ok(event) = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
            if event.is_some() {
                event_count += 1;
            } else {
                break;
            }
        }

        // Should have received fewer events than modifications due to coalescing
        assert!(event_count > 0 && event_count <= 2, "Expected 1-2 events, got {}", event_count);

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
