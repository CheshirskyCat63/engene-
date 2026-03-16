//! Phase 11.5: Surface hydrology lite -- water accumulation, runoff, drainage.

use glam::Vec2;

pub struct HydrologyConfig {
    pub active_radius: f32,
    pub cell_size: f32,
    pub evaporation_rate: f32,
    pub max_memory_bytes: usize,
}

impl Default for HydrologyConfig {
    fn default() -> Self {
        Self {
            active_radius: 64.0,
            cell_size: 0.5,
            evaporation_rate: 0.001,
            max_memory_bytes: 2 * 1024 * 1024,
        }
    }
}

/// Single hydrology cell with water level and flow rates.
#[derive(Clone, Debug)]
pub struct HydrologyCell {
    pub water_level: f32,
    pub absorption_rate: f32,
    pub drainage_rate: f32,
}

impl Default for HydrologyCell {
    fn default() -> Self {
        Self {
            water_level: 0.0,
            absorption_rate: 0.1,
            drainage_rate: 0.05,
        }
    }
}

/// Grid of hydrology cells with compressed water levels.
pub struct HydrologyGrid {
    pub grid: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub center: Vec2,
    pub evaporation_rate: f32,
}

impl HydrologyGrid {
    pub fn new(config: &HydrologyConfig) -> Self {
        let extent = (config.active_radius * 2.0 / config.cell_size) as u32;
        let width = extent.max(1);
        let height = extent.max(1);
        let len = (width * height) as usize;
        Self {
            grid: vec![0; len],
            width,
            height,
            cell_size: config.cell_size,
            center: Vec2::ZERO,
            evaporation_rate: config.evaporation_rate,
        }
    }

    pub fn update(
        &mut self,
        rain_intensity: f32,
        _wind_speed: f32,
        temperature: f32,
        humidity: f32,
        dt: f32,
    ) {
        let incoming = rain_intensity * dt;
        let evaporated = self.evaporation_rate * temperature * (1.0 - humidity) * dt;

        let w = self.width as i32;
        let h = self.height as i32;
        let mut next = vec![0u8; self.grid.len()];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let water = self.grid[idx] as f32 / 255.0;
                let after_rain = water + incoming;
                let after_evap = (after_rain - evaporated).max(0.0);
                let mut cell = after_evap;

                // Simple flow to lower neighbors
                let my_height = 1.0 - cell;
                for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx >= 0 && nx < w && ny >= 0 && ny < h {
                        let nidx = (ny * w + nx) as usize;
                        let nwater = self.grid[nidx] as f32 / 255.0;
                        let nheight = 1.0 - nwater;
                        if nheight > my_height {
                            let flow = 0.05 * cell * dt;
                            cell = (cell - flow).max(0.0);
                        }
                    }
                }
                next[idx] = (cell.min(1.0) * 255.0) as u8;
            }
        }
        self.grid = next;
    }

    pub fn water_level_at(&self, world_pos: Vec2) -> f32 {
        let local = world_pos - self.center;
        let half_extent_x = (self.width as f32 * self.cell_size) * 0.5;
        let half_extent_y = (self.height as f32 * self.cell_size) * 0.5;
        let cell_x = (local.x + half_extent_x) / self.cell_size;
        let cell_y = (local.y + half_extent_y) / self.cell_size;
        let ix = cell_x as i32;
        let iy = cell_y as i32;

        if ix < 0 || iy < 0 {
            return 0.0;
        }
        let ux = ix as u32;
        let uy = iy as u32;
        if ux >= self.width || uy >= self.height {
            return 0.0;
        }

        let idx = (uy * self.width + ux) as usize;
        (self.grid[idx] as f32) / 255.0
    }

    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.grid.len()
    }
}
