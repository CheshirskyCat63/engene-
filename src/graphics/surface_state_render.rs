use crate::world::surface_state::SurfaceStateStore;

pub struct SurfaceStateRenderSystem {
    pub dirty_upload_count: usize,
}

impl SurfaceStateRenderSystem {
    pub fn new() -> Self {
        Self {
            dirty_upload_count: 0,
        }
    }

    pub fn update(&mut self, store: &mut SurfaceStateStore) {
        let dirty = store.drain_dirty_batch();
        self.dirty_upload_count = dirty.len();
    }
}
