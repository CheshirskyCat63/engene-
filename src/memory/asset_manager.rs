use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;

use engine_render::model_loader::{self, LoadedModel};

pub type AssetHandle = u64;

enum AssetRequest {
    LoadModel(AssetHandle, PathBuf),
    LoadTexture(AssetHandle, PathBuf),
    Shutdown,
}

enum AssetResult {
    ModelReady(AssetHandle, Result<LoadedModel, String>),
    TextureReady(AssetHandle, Result<Vec<u8>, String>),
}

pub enum AssetState<T> {
    Loading,
    Ready(T),
    Failed(String),
}

pub struct AssetManager {
    next_handle: AssetHandle,
    models: HashMap<AssetHandle, AssetState<Arc<LoadedModel>>>,
    textures: HashMap<AssetHandle, AssetState<Vec<u8>>>,
    path_to_handle: HashMap<PathBuf, AssetHandle>,
    texture_path_to_handle: HashMap<PathBuf, AssetHandle>,
    tx: mpsc::Sender<AssetRequest>,
    rx_results: mpsc::Receiver<AssetResult>,
    _worker: std::thread::JoinHandle<()>,
}

impl AssetManager {
    pub fn new() -> Self {
        let (tx, rx_req) = mpsc::channel::<AssetRequest>();
        let (tx_res, rx_results) = mpsc::channel::<AssetResult>();

        let worker = std::thread::Builder::new()
            .name("asset-loader".into())
            .spawn(move || {
                while let Ok(req) = rx_req.recv() {
                    match req {
                        AssetRequest::LoadModel(handle, path) => {
                            let result = model_loader::load_glb(&path);
                            let _ = tx_res.send(AssetResult::ModelReady(handle, result));
                        }
                        AssetRequest::LoadTexture(handle, path) => {
                            let result = std::fs::read(&path).map_err(|e| e.to_string());
                            let _ = tx_res.send(AssetResult::TextureReady(handle, result));
                        }
                        AssetRequest::Shutdown => break,
                    }
                }
            })
            .expect("failed to spawn asset loader thread");

        Self {
            next_handle: 1,
            models: HashMap::new(),
            textures: HashMap::new(),
            path_to_handle: HashMap::new(),
            texture_path_to_handle: HashMap::new(),
            tx,
            rx_results,
            _worker: worker,
        }
    }

    pub fn request_model(&mut self, path: &Path) -> AssetHandle {
        let canonical = path.to_path_buf();
        if let Some(&existing) = self.path_to_handle.get(&canonical) {
            return existing;
        }
        let handle = self.next_handle;
        self.next_handle += 1;
        self.models.insert(handle, AssetState::Loading);
        self.path_to_handle.insert(canonical.clone(), handle);
        let _ = self.tx.send(AssetRequest::LoadModel(handle, canonical));
        handle
    }

    pub fn request_texture(&mut self, path: &Path) -> AssetHandle {
        let canonical = path.to_path_buf();
        if let Some(&existing) = self.texture_path_to_handle.get(&canonical) {
            return existing;
        }
        let handle = self.next_handle;
        self.next_handle += 1;
        self.textures.insert(handle, AssetState::Loading);
        self.texture_path_to_handle
            .insert(canonical.clone(), handle);
        let _ = self.tx.send(AssetRequest::LoadTexture(handle, canonical));
        handle
    }

    pub fn poll(&mut self) {
        while let Ok(result) = self.rx_results.try_recv() {
            match result {
                AssetResult::ModelReady(handle, Ok(model)) => {
                    self.models
                        .insert(handle, AssetState::Ready(Arc::new(model)));
                }
                AssetResult::ModelReady(handle, Err(e)) => {
                    tracing::warn!("asset load failed for handle {}: {}", handle, e);
                    self.models.insert(handle, AssetState::Failed(e));
                }
                AssetResult::TextureReady(handle, Ok(bytes)) => {
                    self.textures.insert(handle, AssetState::Ready(bytes));
                }
                AssetResult::TextureReady(handle, Err(e)) => {
                    tracing::warn!("texture load failed for handle {}: {}", handle, e);
                    self.textures.insert(handle, AssetState::Failed(e));
                }
            }
        }
    }

    pub fn get_model(&self, handle: AssetHandle) -> Option<&AssetState<Arc<LoadedModel>>> {
        self.models.get(&handle)
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(AssetRequest::Shutdown);
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
