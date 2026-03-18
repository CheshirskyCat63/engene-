use std::collections::{HashMap, VecDeque};

use crate::physics::damage_pipeline::response_aggregator::SurfaceMaskType;

const MAX_ACTIVE_PATCHES: usize = 4096;
const MAX_MEMORY_BYTES: usize = 32 * 1024 * 1024;
const MAX_DIRTY_UPLOADS_PER_FRAME: usize = 16;
const MAX_UPDATES_PER_FRAME: usize = 32;

#[derive(Clone, Debug)]
pub struct SurfaceStatePatch {
    pub cell_x: u32,
    pub cell_z: u32,
    pub dirt_mask: f32,
    pub wetness_mask: f32,
    pub scorch_mask: f32,
    pub wear_mask: f32,
    pub blood_stain: f32,
    pub blood_stain_age: f32,
    pub impact_density: f32,
    pub crack_persistence: f32,
}

impl SurfaceStatePatch {
    pub fn new(cell_x: u32, cell_z: u32) -> Self {
        Self {
            cell_x,
            cell_z,
            dirt_mask: 0.0,
            wetness_mask: 0.0,
            scorch_mask: 0.0,
            wear_mask: 0.0,
            blood_stain: 0.0,
            blood_stain_age: 0.0,
            impact_density: 0.0,
            crack_persistence: 0.0,
        }
    }

    pub fn is_negligible(&self) -> bool {
        self.dirt_mask < 0.01
            && self.wetness_mask < 0.01
            && self.scorch_mask < 0.01
            && self.wear_mask < 0.01
            && self.blood_stain < 0.01
            && self.impact_density < 0.01
            && self.crack_persistence < 0.01
    }
}

#[derive(Clone, Debug)]
pub struct CompressedSurfacePatch {
    pub cell_x: u32,
    pub cell_z: u32,
    pub combined_damage: u8,
    pub blood_present: bool,
}

impl CompressedSurfacePatch {
    pub fn from_full(full: &SurfaceStatePatch) -> Self {
        let wear_bits = ((full.wear_mask * 15.0).min(15.0) as u8) & 0x0F;
        let scorch_bits = (((full.scorch_mask * 15.0).min(15.0) as u8) & 0x0F) << 4;
        Self {
            cell_x: full.cell_x,
            cell_z: full.cell_z,
            combined_damage: wear_bits | scorch_bits,
            blood_present: full.blood_stain > 0.1,
        }
    }
}

pub struct SurfaceStateStore {
    active_patches: HashMap<(u32, u32), SurfaceStatePatch>,
    compressed_patches: HashMap<(u32, u32), CompressedSurfacePatch>,
    dirty_patches: Vec<(u32, u32)>,
    eviction_queue: VecDeque<(u32, u32)>,
}

impl SurfaceStateStore {
    pub fn new() -> Self {
        Self {
            active_patches: HashMap::new(),
            compressed_patches: HashMap::new(),
            dirty_patches: Vec::new(),
            eviction_queue: VecDeque::new(),
        }
    }

    pub fn apply_mask_delta(
        &mut self,
        cell_x: u32,
        cell_z: u32,
        mask: SurfaceMaskType,
        delta: f32,
    ) {
        let key = (cell_x, cell_z);
        let patch = self.active_patches.entry(key).or_insert_with(|| {
            self.eviction_queue.push_back(key);
            SurfaceStatePatch::new(cell_x, cell_z)
        });

        match mask {
            SurfaceMaskType::Dirt => patch.dirt_mask = (patch.dirt_mask + delta).clamp(0.0, 1.0),
            SurfaceMaskType::Wetness => {
                patch.wetness_mask = (patch.wetness_mask + delta).clamp(0.0, 1.0)
            }
            SurfaceMaskType::Scorch => {
                patch.scorch_mask = (patch.scorch_mask + delta).clamp(0.0, 1.0)
            }
            SurfaceMaskType::Wear => patch.wear_mask = (patch.wear_mask + delta).clamp(0.0, 1.0),
            SurfaceMaskType::BloodStain => {
                patch.blood_stain = (patch.blood_stain + delta).clamp(0.0, 1.0);
                if delta > 0.0 {
                    patch.blood_stain_age = 0.0;
                }
            }
            SurfaceMaskType::ImpactDensity => {
                patch.impact_density = (patch.impact_density + delta).clamp(0.0, 1.0);
                if patch.impact_density > 0.8 {
                    patch.crack_persistence = (patch.crack_persistence + 0.1).min(1.0);
                }
            }
            SurfaceMaskType::CrackPersistence => {
                patch.crack_persistence = (patch.crack_persistence + delta).clamp(0.0, 1.0);
            }
        }

        if !self.dirty_patches.contains(&key) {
            self.dirty_patches.push(key);
        }

        self.enforce_memory_cap();
    }

    pub fn decay_tick(&mut self, dt: f32, raining: bool) {
        let rain_mult = if raining { 2.0 } else { 1.0 };
        let mut updates = 0;

        for patch in self.active_patches.values_mut() {
            if updates >= MAX_UPDATES_PER_FRAME {
                break;
            }
            updates += 1;

            patch.dirt_mask -= dt / 600.0 * rain_mult;
            patch.wetness_mask -= dt / 120.0;
            if raining {
                patch.wetness_mask += dt * 0.01;
            }
            patch.blood_stain_age += dt;
            if patch.blood_stain_age > 1800.0 {
                patch.blood_stain -= dt / 7200.0 * rain_mult;
            }
            if patch.impact_density > 0.0 && patch.impact_density < 0.8 {
                patch.impact_density -= dt / 3600.0;
            }

            patch.dirt_mask = patch.dirt_mask.max(0.0);
            patch.wetness_mask = patch.wetness_mask.clamp(0.0, 1.0);
            patch.blood_stain = patch.blood_stain.max(0.0);
            patch.impact_density = patch.impact_density.max(0.0);
        }

        self.active_patches.retain(|_, p| !p.is_negligible());
    }

    pub fn drain_dirty_batch(&mut self) -> Vec<(u32, u32)> {
        let count = self.dirty_patches.len().min(MAX_DIRTY_UPLOADS_PER_FRAME);
        self.dirty_patches.drain(..count).collect()
    }

    pub fn estimated_memory(&self) -> usize {
        self.active_patches.len() * std::mem::size_of::<SurfaceStatePatch>()
            + self.compressed_patches.len() * 64
    }

    fn enforce_memory_cap(&mut self) {
        while self.active_patches.len() > MAX_ACTIVE_PATCHES
            || self.estimated_memory() > MAX_MEMORY_BYTES
        {
            if let Some(key) = self.eviction_queue.pop_front() {
                if let Some(full) = self.active_patches.remove(&key) {
                    let compressed = CompressedSurfacePatch::from_full(&full);
                    self.compressed_patches.insert(key, compressed);
                }
            } else {
                break;
            }
        }
    }

    pub fn get(&self, cx: u32, cz: u32) -> Option<&SurfaceStatePatch> {
        self.active_patches.get(&(cx, cz))
    }

    pub fn active_count(&self) -> usize {
        self.active_patches.len()
    }

    pub fn compressed_count(&self) -> usize {
        self.compressed_patches.len()
    }
}
