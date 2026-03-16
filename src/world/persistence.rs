use crate::world::cell::{CELL_SIZE, GRID_SIZE};

pub const TERRAIN_LOD_LEVELS: usize = 3;

pub struct TerrainChunk {
    pub chunk_x: u32,
    pub chunk_z: u32,
    pub lod: u8,
    pub vertex_count: u32,
    pub loaded: bool,
}

pub struct ChunkTerrainLod {
    pub chunks_per_side: u32,
    pub chunk_world_size: f32,
    pub chunks: Vec<TerrainChunk>,
}

impl ChunkTerrainLod {
    pub fn new(chunks_per_side: u32) -> Self {
        let chunk_world_size = (GRID_SIZE as f32 * CELL_SIZE) / chunks_per_side as f32;
        let mut chunks = Vec::with_capacity((chunks_per_side * chunks_per_side) as usize);
        for cz in 0..chunks_per_side {
            for cx in 0..chunks_per_side {
                chunks.push(TerrainChunk {
                    chunk_x: cx,
                    chunk_z: cz,
                    lod: 0,
                    vertex_count: 0,
                    loaded: false,
                });
            }
        }
        Self {
            chunks_per_side,
            chunk_world_size,
            chunks,
        }
    }

    pub fn update_lods(&mut self, camera_x: f32, camera_z: f32) {
        let lod_distances = [
            self.chunk_world_size * 3.0,
            self.chunk_world_size * 8.0,
            self.chunk_world_size * 20.0,
        ];

        for chunk in &mut self.chunks {
            let cx = (chunk.chunk_x as f32 + 0.5) * self.chunk_world_size;
            let cz = (chunk.chunk_z as f32 + 0.5) * self.chunk_world_size;
            let dist = ((cx - camera_x).powi(2) + (cz - camera_z).powi(2)).sqrt();

            chunk.lod = if dist < lod_distances[0] {
                0
            } else if dist < lod_distances[1] {
                1
            } else if dist < lod_distances[2] {
                2
            } else {
                2
            };
        }
    }

    pub fn resolution_for_lod(lod: u8) -> u32 {
        match lod {
            0 => 32,
            1 => 16,
            _ => 8,
        }
    }

    pub fn geomorph_factor(dist: f32, lod_near: f32, lod_far: f32) -> f32 {
        ((dist - lod_near) / (lod_far - lod_near)).clamp(0.0, 1.0)
    }

    pub fn chunks_in_range(&self, cam_x: f32, cam_z: f32, radius: f32) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        for chunk in &self.chunks {
            let cx = (chunk.chunk_x as f32 + 0.5) * self.chunk_world_size;
            let cz = (chunk.chunk_z as f32 + 0.5) * self.chunk_world_size;
            let dist = ((cx - cam_x).powi(2) + (cz - cam_z).powi(2)).sqrt();
            if dist < radius {
                result.push((chunk.chunk_x, chunk.chunk_z));
            }
        }
        result
    }
}

pub struct VegetationCluster {
    pub chunk_x: u32,
    pub chunk_z: u32,
    pub tree_count: u32,
    pub grass_count: u32,
    pub loaded: bool,
}

pub struct VegetationStreaming {
    pub clusters: Vec<VegetationCluster>,
    pub load_radius: f32,
    pub unload_radius: f32,
}

impl VegetationStreaming {
    pub fn new(chunks_per_side: u32, load_radius: f32, unload_radius: f32) -> Self {
        let mut clusters = Vec::with_capacity((chunks_per_side * chunks_per_side) as usize);
        for cz in 0..chunks_per_side {
            for cx in 0..chunks_per_side {
                clusters.push(VegetationCluster {
                    chunk_x: cx,
                    chunk_z: cz,
                    tree_count: 0,
                    grass_count: 0,
                    loaded: false,
                });
            }
        }
        Self {
            clusters,
            load_radius,
            unload_radius,
        }
    }

    pub fn update(&mut self, cam_x: f32, cam_z: f32, chunk_size: f32) -> (Vec<usize>, Vec<usize>) {
        let mut to_load = Vec::new();
        let mut to_unload = Vec::new();

        for (i, cluster) in self.clusters.iter_mut().enumerate() {
            let cx = (cluster.chunk_x as f32 + 0.5) * chunk_size;
            let cz = (cluster.chunk_z as f32 + 0.5) * chunk_size;
            let dist = ((cx - cam_x).powi(2) + (cz - cam_z).powi(2)).sqrt();

            if !cluster.loaded && dist < self.load_radius {
                cluster.loaded = true;
                to_load.push(i);
            } else if cluster.loaded && dist > self.unload_radius {
                cluster.loaded = false;
                to_unload.push(i);
            }
        }

        (to_load, to_unload)
    }
}
