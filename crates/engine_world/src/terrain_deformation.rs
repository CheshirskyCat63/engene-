use std::collections::{HashMap, VecDeque};

use crate::world::terrain_damage::CraterStamp;

const MAX_STAMPS_PER_FRAME: usize = 4;
const MAX_MODIFIED_PATCHES: usize = 1024;
const PATCH_SIZE: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct TerrainPatch {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub height_deltas: Vec<f32>,
    pub dirty: bool,
}

pub struct TerrainDeformationSystem {
    pending_stamps: VecDeque<CraterStamp>,
    modified_patches: HashMap<(i32, i32), TerrainPatch>,
    total_memory_bytes: usize,
}

impl TerrainDeformationSystem {
    pub fn new() -> Self {
        Self {
            pending_stamps: VecDeque::new(),
            modified_patches: HashMap::new(),
            total_memory_bytes: 0,
        }
    }

    pub fn submit_crater(&mut self, stamp: CraterStamp) {
        self.pending_stamps.push_back(stamp);
    }

    pub fn process_frame(&mut self) {
        let count = self.pending_stamps.len().min(MAX_STAMPS_PER_FRAME);
        for _ in 0..count {
            if let Some(stamp) = self.pending_stamps.pop_front() {
                self.apply_stamp(&stamp);
            }
        }
        self.update_memory_estimate();
    }

    fn apply_stamp(&mut self, stamp: &CraterStamp) {
        let cx = (stamp.center.x / PATCH_SIZE).floor() as i32;
        let cz = (stamp.center.z / PATCH_SIZE).floor() as i32;
        let radius_in_patches = (stamp.radius / PATCH_SIZE).ceil() as i32 + 1;

        for dx in -radius_in_patches..=radius_in_patches {
            for dz in -radius_in_patches..=radius_in_patches {
                let key = (cx + dx, cz + dz);
                if self.modified_patches.len() >= MAX_MODIFIED_PATCHES
                    && !self.modified_patches.contains_key(&key)
                {
                    continue;
                }

                let patch = self
                    .modified_patches
                    .entry(key)
                    .or_insert_with(|| TerrainPatch {
                        chunk_x: key.0,
                        chunk_z: key.1,
                        height_deltas: vec![0.0; 64],
                        dirty: false,
                    });

                let patch_world_x = key.0 as f32 * PATCH_SIZE;
                let patch_world_z = key.1 as f32 * PATCH_SIZE;
                let dist = ((patch_world_x + PATCH_SIZE * 0.5 - stamp.center.x).powi(2)
                    + (patch_world_z + PATCH_SIZE * 0.5 - stamp.center.z).powi(2))
                .sqrt();

                let delta = stamp.sample_depth(dist);
                if delta.abs() > 0.01 {
                    for h in patch.height_deltas.iter_mut() {
                        *h += delta;
                    }
                    patch.dirty = true;
                }
            }
        }
    }

    pub fn drain_dirty_patches(&mut self) -> Vec<(i32, i32)> {
        let mut dirty = Vec::new();
        for (key, patch) in &mut self.modified_patches {
            if patch.dirty {
                dirty.push(*key);
                patch.dirty = false;
            }
        }
        dirty
    }

    pub fn patch_count(&self) -> usize {
        self.modified_patches.len()
    }

    pub fn memory_usage(&self) -> usize {
        self.total_memory_bytes
    }
}

impl TerrainDeformationSystem {
    fn update_memory_estimate(&mut self) {
        self.total_memory_bytes =
            self.modified_patches.len() * (std::mem::size_of::<TerrainPatch>() + 64 * 4);
    }
}
