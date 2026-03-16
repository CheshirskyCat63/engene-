//! Hot Reload System for Shaders and Configs (Phase D.1/D.2)
//! 
//! Provides file watching and hot reload capabilities for development.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// Hot reload event
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    ShaderChanged { path: PathBuf },
    ConfigChanged { path: PathBuf },
    AssetChanged { path: PathBuf },
}

/// Hot reload manager
pub struct HotReloadManager {
    /// Watched paths and their last modification times
    watched_paths: HashMap<PathBuf, Instant>,
    /// Event sender
    sender: Sender<HotReloadEvent>,
    /// Event receiver
    receiver: Receiver<HotReloadEvent>,
    /// Poll interval
    poll_interval: Duration,
    /// Enabled flag
    enabled: bool,
    /// Shader directories
    shader_dirs: Vec<PathBuf>,
    /// Config directories  
    config_dirs: Vec<PathBuf>,
}

impl HotReloadManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            watched_paths: HashMap::new(),
            sender,
            receiver,
            poll_interval: Duration::from_millis(100),
            enabled: true,
            shader_dirs: vec![PathBuf::from("assets/shaders")],
            config_dirs: vec![PathBuf::from("game/data")],
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn add_shader_dir(&mut self, path: impl Into<PathBuf>) {
        self.shader_dirs.push(path.into());
    }

    pub fn add_config_dir(&mut self, path: impl Into<PathBuf>) {
        self.config_dirs.push(path.into());
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Poll for changes (call from main loop)
    pub fn poll(&mut self) -> Vec<HotReloadEvent> {
        if !self.enabled {
            return Vec::new();
        }

        let mut events = Vec::new();

        // Collect paths first to avoid borrow conflicts
        let shader_paths: Vec<PathBuf> = self.shader_dirs.iter()
            .flat_map(|dir| {
                std::fs::read_dir(dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "wgsl"))
            })
            .collect();

        let config_paths: Vec<PathBuf> = self.config_dirs.iter()
            .flat_map(|dir| {
                std::fs::read_dir(dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "ron" || ext == "json"))
            })
            .collect();

        // Check shader files
        for path in shader_paths {
            if let Some(event) = self.check_file(&path, HotReloadEvent::ShaderChanged { path: path.clone() }) {
                events.push(event);
            }
        }

        // Check config files
        for path in config_paths {
            if let Some(event) = self.check_file(&path, HotReloadEvent::ConfigChanged { path: path.clone() }) {
                events.push(event);
            }
        }

        // Also check for events from other threads
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }

        events
    }

    fn check_file(&mut self, path: &Path, event: HotReloadEvent) -> Option<HotReloadEvent> {
        let metadata = std::fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        let instant = Instant::now() - modified.elapsed().ok()?;
        
        let last_modified = self.watched_paths.get(path).copied();
        self.watched_paths.insert(path.to_path_buf(), instant);

        match last_modified {
            Some(last) if instant > last => Some(event),
            None => Some(event), // First check
            _ => None,
        }
    }

    /// Get sender for external file watchers
    pub fn sender(&self) -> Sender<HotReloadEvent> {
        self.sender.clone()
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shader hot reloader
pub struct ShaderHotReloader {
    /// Cached shader source
    shader_cache: HashMap<PathBuf, String>,
    /// Compiled shader modules (device-specific)
    compiled_modules: HashMap<PathBuf, wgpu::ShaderModule>,
}

impl ShaderHotReloader {
    pub fn new() -> Self {
        Self {
            shader_cache: HashMap::new(),
            compiled_modules: HashMap::new(),
        }
    }

    /// Load shader from file or cache
    pub fn load_shader(&mut self, path: &Path) -> std::io::Result<String> {
        if let Some(cached) = self.shader_cache.get(path) {
            return Ok(cached.clone());
        }

        let source = std::fs::read_to_string(path)?;
        self.shader_cache.insert(path.to_path_buf(), source.clone());
        Ok(source)
    }

    /// Reload shader from file (ignores cache)
    pub fn reload_shader(&mut self, path: &Path) -> std::io::Result<String> {
        let source = std::fs::read_to_string(path)?;
        self.shader_cache.insert(path.to_path_buf(), source.clone());
        Ok(source)
    }

    /// Create shader module from file
    pub fn create_shader_module(
        &mut self,
        device: &wgpu::Device,
        path: &Path,
        label: Option<&str>,
    ) -> wgpu::ShaderModule {
        let source = self.load_shader(path).expect("Failed to load shader");
        
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        self.compiled_modules.insert(path.to_path_buf(), module.clone());
        module
    }

    /// Get cached shader source
    pub fn get_cached(&self, path: &Path) -> Option<&String> {
        self.shader_cache.get(path)
    }

    /// Clear cache (forces reload on next access)
    pub fn clear_cache(&mut self) {
        self.shader_cache.clear();
        self.compiled_modules.clear();
    }
}

impl Default for ShaderHotReloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Config hot reloader
pub struct ConfigHotReloader {
    /// Cached config content
    config_cache: HashMap<PathBuf, Vec<u8>>,
    /// Last reload time
    last_reload: HashMap<PathBuf, Instant>,
}

impl ConfigHotReloader {
    pub fn new() -> Self {
        Self {
            config_cache: HashMap::new(),
            last_reload: HashMap::new(),
        }
    }

    /// Load config from file or cache
    pub fn load_config(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        if let Some(cached) = self.config_cache.get(path) {
            return Ok(cached.clone());
        }

        let data = std::fs::read(path)?;
        self.config_cache.insert(path.to_path_buf(), data.clone());
        self.last_reload.insert(path.to_path_buf(), Instant::now());
        Ok(data)
    }

    /// Reload config from file (ignores cache)
    pub fn reload_config(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let data = std::fs::read(path)?;
        self.config_cache.insert(path.to_path_buf(), data.clone());
        self.last_reload.insert(path.to_path_buf(), Instant::now());
        Ok(data)
    }

    /// Parse config as RON
    pub fn load_ron<T: serde::de::DeserializeOwned>(&mut self, path: &Path) -> Result<T, Box<dyn std::error::Error>> {
        let data = self.load_config(path)?;
        let config: T = ron::de::from_bytes(&data)?;
        Ok(config)
    }

    /// Parse config as JSON (requires serde_json)
    pub fn load_json<T: serde::de::DeserializeOwned>(&mut self, path: &Path) -> Result<T, Box<dyn std::error::Error>> {
        let data = self.load_config(path)?;
        
        // Try JSON first, fall back to RON
        if let Ok(config) = serde_json::from_slice::<T>(&data) {
            return Ok(config);
        }
        
        // Try RON as fallback
        let config: T = ron::de::from_bytes(&data)?;
        Ok(config)
    }

    /// Get last reload time
    pub fn last_reload(&self, path: &Path) -> Option<Instant> {
        self.last_reload.get(path).copied()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.config_cache.clear();
        self.last_reload.clear();
    }
}

impl Default for ConfigHotReloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined hot reload system
pub struct HotReloadSystem {
    pub manager: HotReloadManager,
    pub shaders: ShaderHotReloader,
    pub configs: ConfigHotReloader,
}

impl HotReloadSystem {
    pub fn new() -> Self {
        Self {
            manager: HotReloadManager::new(),
            shaders: ShaderHotReloader::new(),
            configs: ConfigHotReloader::new(),
        }
    }

    /// Process hot reload events
    pub fn update(&mut self) -> Vec<HotReloadEvent> {
        self.manager.poll()
    }

    /// Handle a hot reload event
    pub fn handle_event(&mut self, event: &HotReloadEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            HotReloadEvent::ShaderChanged { path } => {
                self.shaders.reload_shader(path)?;
                println!("[HotReload] Shader reloaded: {}", path.display());
            }
            HotReloadEvent::ConfigChanged { path } => {
                self.configs.reload_config(path)?;
                println!("[HotReload] Config reloaded: {}", path.display());
            }
            HotReloadEvent::AssetChanged { path } => {
                println!("[HotReload] Asset changed: {}", path.display());
            }
        }
        Ok(())
    }
}

impl Default for HotReloadSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_manager() {
        let manager = HotReloadManager::new();
        assert!(manager.enabled);
    }

    #[test]
    fn test_shader_hot_reloader() {
        let mut reloader = ShaderHotReloader::new();
        
        // Non-existent file
        let result = reloader.load_shader(Path::new("nonexistent.wgsl"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_hot_reloader() {
        let mut reloader = ConfigHotReloader::new();
        
        // Non-existent file
        let result = reloader.load_config(Path::new("nonexistent.ron"));
        assert!(result.is_err());
    }

    #[test]
    fn test_hot_reload_system() {
        let system = HotReloadSystem::new();
        assert!(system.manager.enabled);
    }
}
