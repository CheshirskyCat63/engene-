use crate::world::biome::Biome;
use crate::world::cell::{CELL_SIZE, GRID_SIZE, WORLD_SIZE};

pub struct Heightmap {
    pub data: Vec<f32>,
    pub resolution: u32,
    pub world_size: f32,
}

impl Heightmap {
    /// Flat heightmap for bounded test arenas (e.g. Destruction Sandbox 50x50).
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

    pub fn generate(biomes: &[Biome]) -> Self {
        let resolution = (GRID_SIZE * 8).min(1024);
        let world_size = WORLD_SIZE;
        let grid_size = GRID_SIZE;
        let n = (resolution + 1) as usize;
        let mut data = vec![0.0f32; n * n];

        for iz in 0..=resolution {
            for ix in 0..=resolution {
                let wx = ix as f32 / resolution as f32 * world_size;
                let wz = iz as f32 / resolution as f32 * world_size;

                let base = fbm(wx * 0.003, wz * 0.003, 6) * 80.0;

                let cx = ((wx / CELL_SIZE) as u32).min(grid_size - 1);
                let cy = ((wz / CELL_SIZE) as u32).min(grid_size - 1);
                let biome_idx = (cy * grid_size + cx) as usize;
                let biome = if biome_idx < biomes.len() {
                    biomes[biome_idx]
                } else {
                    Biome::Plains
                };

                let height = match biome {
                    Biome::Hills => base * 1.6 + 20.0,
                    Biome::Forest => base * 0.8 + 5.0,
                    Biome::Plains => base * 0.4,
                    Biome::Swamp => base * 0.15 - 2.0,
                    Biome::Settlement => base * 0.3 + 2.0,
                };

                data[iz as usize * n + ix as usize] = height;
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
        let sx =
            (x / self.world_size * self.resolution as f32).clamp(0.0, (self.resolution - 1) as f32);
        let sz =
            (z / self.world_size * self.resolution as f32).clamp(0.0, (self.resolution - 1) as f32);

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

    pub fn normal_at(&self, x: f32, z: f32) -> [f32; 3] {
        let eps = self.world_size / self.resolution as f32;
        let hx0 = self.sample(x - eps, z);
        let hx1 = self.sample(x + eps, z);
        let hz0 = self.sample(x, z - eps);
        let hz1 = self.sample(x, z + eps);

        let dx = (hx1 - hx0) / (2.0 * eps);
        let dz = (hz1 - hz0) / (2.0 * eps);
        let len = (dx * dx + 1.0 + dz * dz).sqrt();
        [-dx / len, 1.0 / len, -dz / len]
    }
}

fn hash2d(ix: i32, iy: i32) -> f32 {
    let mut n = ix
        .wrapping_mul(374761393)
        .wrapping_add(iy.wrapping_mul(668265263));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n = n ^ (n >> 16);
    (n & 0x7fffffff) as f32 / 0x7fffffff as f32
}

fn smooth_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let fx = fx * fx * (3.0 - 2.0 * fx);
    let fy = fy * fy * (3.0 - 2.0 * fy);

    let a = hash2d(ix, iy);
    let b = hash2d(ix + 1, iy);
    let c = hash2d(ix, iy + 1);
    let d = hash2d(ix + 1, iy + 1);

    let ab = a + fx * (b - a);
    let cd = c + fx * (d - c);
    ab + fy * (cd - ab)
}

fn fbm(x: f32, y: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_amp = 0.0f32;

    for _ in 0..octaves {
        value += amplitude * smooth_noise(x * frequency, y * frequency);
        max_amp += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_amp
}
