// Heightmap - terrain height data
// Migrated from root src/world/heightmap.rs

use crate::cell::{CELL_SIZE, GRID_SIZE, WORLD_SIZE};

pub struct Heightmap {
    pub data: Vec<f32>,
    pub resolution: u32,
    pub world_size: f32,
}

impl Heightmap {
    /// Flat heightmap for bounded test arenas (e.g. Destruction Tools 50x50).
    pub fn flat(world_size: f32) -> Self {
        let resolution = 128u32;
        let n = (resolution + 1) as usize;
        let data = vec![0.0f32; n * n];
        Self {
            data,
            resolution,
            world_size,
        }
    }

    /// Generate a procedural heightmap
    pub fn generate() -> Self {
        let resolution = (GRID_SIZE * 8).min(1024);
        let world_size = WORLD_SIZE;
        let grid_size = GRID_SIZE;
        let n = (resolution + 1) as usize;
        let mut data = vec![0.0f32; n * n];

        for iz in 0..=resolution {
            for ix in 0..=resolution {
                let wx = ix as f32 / resolution as f32 * world_size;
                let wz = iz as f32 / resolution as f32 * world_size;

                // Simplified procedural height
                let base = (wx * 0.003).sin() * (wz * 0.003).cos() * 40.0 + 20.0;
                data[iz as usize * n + ix as usize] = base;
            }
        }

        Self {
            data,
            resolution,
            world_size,
        }
    }

    pub fn sample(&self, x: f32, z: f32) -> f32 {
        let n = self.resolution + 1;
        let sx = (x / self.world_size * self.resolution as f32).clamp(0.0, (self.resolution - 1) as f32);
        let sz = (z / self.world_size * self.resolution as f32).clamp(0.0, (self.resolution - 1) as f32);

        let ix = sx as u32;
        let iz = sz as u32;
        let fx = sx - ix as f32;
        let fz = sz - iz as f32;

        let i00 = iz * n + ix;
        let i10 = iz * n + (ix + 1).min(self.resolution);
        let i01 = (iz + 1).min(self.resolution) * n + ix;
        let i11 = (iz + 1).min(self.resolution) * n + (ix + 1).min(self.resolution);

        let h00 = self.data[i00 as usize];
        let h10 = self.data[i10 as usize];
        let h01 = self.data[i01 as usize];
        let h11 = self.data[i11 as usize];

        let a = h00 + fx * (h10 - h00);
        let b = h01 + fx * (h11 - h01);
        a + fz * (b - a)
    }
}

impl Default for Heightmap {
    fn default() -> Self {
        Self::flat(1000.0)
    }
}
