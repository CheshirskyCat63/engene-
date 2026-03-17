//! Hot reload system for shaders and config files (Phase D.2)
//! 
//! Watches external files and triggers reload callbacks on change.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

/// File event for hot reload
#[derive(Debug, Clone)]
pub enum FileEvent {
    Modified(PathBuf),
    Created(PathBuf),
    Removed(PathBuf),
}

/// Hot reload watcher
pub struct HotReloadWatcher {
    /// Watched paths
    watched: HashMap<PathBuf, ()>,
    /// Event sender for internal use
    sender: Option<Sender<FileEvent>>,
    /// Poll interval in milliseconds
    poll_interval_ms: u64,
}

impl Default for HotReloadWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HotReloadWatcher {
    pub fn new() -> Self {
        let (tx, _rx) = channel();
        Self {
            watched: HashMap::new(),
            sender: Some(tx),
            poll_interval_ms: 1000,
        }
    }

    /// Set poll interval (default 1000ms)
    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    /// Watch a file or directory for changes
    pub fn watch(&mut self, path: PathBuf) {
        if path.exists() {
            self.watched.insert(path.clone(), ());
            tracing::info!("watching for changes: {:?}", path);
        } else {
            tracing::warn!("cannot watch non-existent path: {:?}", path);
        }
    }

    /// Unwatch a path
    pub fn unwatch(&mut self, path: &PathBuf) {
        self.watched.remove(path);
        tracing::info!("stopped watching: {:?}", path);
    }

    /// Check for file changes (call this in game loop)
    /// Returns true if any files changed
    pub fn poll_changes(&mut self) -> bool {
        let mut changed = false;
        
        for path in self.watched.keys() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let now = std::time::SystemTime::now();
                    // File modified in last 2 seconds - trigger reload
                    if let Ok(duration) = now.duration_since(modified) {
                        if duration.as_secs() < 2 {
                            if let Some(sender) = &self.sender {
                                let _ = sender.send(FileEvent::Modified(path.clone()));
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        
        changed
    }

    /// Get poll interval
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    /// Number of watched paths
    pub fn watched_count(&self) -> usize {
        self.watched.len()
    }
}

/// Reload context - holds state for hot reload
pub struct ReloadContext {
    /// Last reload timestamp
    pub last_reload: std::time::SystemTime,
    /// Files that need reload
    pub pending: Vec<PathBuf>,
}

impl Default for ReloadContext {
    fn default() -> Self {
        Self {
            last_reload: std::time::SystemTime::now(),
            pending: Vec::new(),
        }
    }
}

impl ReloadContext {
    /// Add a file to pending reload queue
    pub fn queue_reload(&mut self, path: PathBuf) {
        if !self.pending.contains(&path) {
            self.pending.push(path);
        }
    }

    /// Check if any files are pending reload
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Clear pending queue
    pub fn clear_pending(&mut self) {
        self.pending.clear();
        self.last_reload = std::time::SystemTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        let watcher = HotReloadWatcher::new();
        assert_eq!(watcher.watched_count(), 0);
    }

    #[test]
    fn test_reload_context() {
        let mut ctx = ReloadContext::default();
        assert!(!ctx.has_pending());
        
        ctx.queue_reload(PathBuf::from("test.wgsl"));
        assert!(ctx.has_pending());
        
        ctx.clear_pending();
        assert!(!ctx.has_pending());
    }
}
