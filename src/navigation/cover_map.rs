use glam::Vec2;

pub struct CoverCell {
    pub quality: f32,
    pub direction: Vec2,
}

pub struct CoverMap {
    cells: Vec<CoverCell>,
    width: usize,
    height: usize,
    cell_size: f32,
    offset_x: f32,
    offset_z: f32,
}

impl CoverMap {
    pub fn precompute(
        world_size: f32,
        cell_size: f32,
        terrain_height_fn: &dyn Fn(f32, f32) -> f32,
    ) -> Self {
        let grid_dim = (world_size / cell_size).ceil() as usize;
        let mut cells = Vec::with_capacity(grid_dim * grid_dim);

        for gz in 0..grid_dim {
            for gx in 0..grid_dim {
                let x = gx as f32 * cell_size + cell_size * 0.5;
                let z = gz as f32 * cell_size + cell_size * 0.5;
                let h_center = terrain_height_fn(x, z);

                let mut max_slope = 0.0_f32;
                let mut best_dir = Vec2::ZERO;
                let probe = cell_size * 0.5;

                for &(dx, dz) in &[(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
                                    (0.7, 0.7), (-0.7, 0.7), (0.7, -0.7), (-0.7, -0.7)] {
                    let nx = x + dx * probe;
                    let nz = z + dz * probe;
                    let nh = terrain_height_fn(nx, nz);
                    let slope = (nh - h_center) / probe;
                    if slope > max_slope {
                        max_slope = slope;
                        best_dir = Vec2::new(-dx, -dz).normalize_or_zero();
                    }
                }

                let quality = (max_slope * 3.0).clamp(0.0, 1.0);
                cells.push(CoverCell { quality, direction: best_dir });
            }
        }

        println!(
            "[cover_map] precomputed {}x{} grid ({} cells, cell_size={:.1}m)",
            grid_dim, grid_dim, cells.len(), cell_size,
        );

        Self {
            cells,
            width: grid_dim,
            height: grid_dim,
            cell_size,
            offset_x: 0.0,
            offset_z: 0.0,
        }
    }

    pub fn get_cover_quality(&self, x: f32, z: f32) -> f32 {
        let gx = ((x - self.offset_x) / self.cell_size) as usize;
        let gz = ((z - self.offset_z) / self.cell_size) as usize;
        if gx >= self.width || gz >= self.height {
            return 0.0;
        }
        self.cells[gz * self.width + gx].quality
    }

    pub fn get_cover_direction(&self, x: f32, z: f32) -> Vec2 {
        let gx = ((x - self.offset_x) / self.cell_size) as usize;
        let gz = ((z - self.offset_z) / self.cell_size) as usize;
        if gx >= self.width || gz >= self.height {
            return Vec2::ZERO;
        }
        self.cells[gz * self.width + gx].direction
    }

    pub fn find_best_cover_near(&self, x: f32, z: f32, radius: f32) -> Option<(f32, f32, f32)> {
        let cells_radius = (radius / self.cell_size).ceil() as i32;
        let gx = ((x - self.offset_x) / self.cell_size) as i32;
        let gz = ((z - self.offset_z) / self.cell_size) as i32;

        let mut best = None;
        let mut best_quality = 0.0;

        for dz in -cells_radius..=cells_radius {
            for dx in -cells_radius..=cells_radius {
                let cx = gx + dx;
                let cz = gz + dz;
                if cx < 0 || cz < 0 || cx >= self.width as i32 || cz >= self.height as i32 {
                    continue;
                }
                let idx = cz as usize * self.width + cx as usize;
                let q = self.cells[idx].quality;
                if q > best_quality {
                    best_quality = q;
                    let wx = cx as f32 * self.cell_size + self.offset_x + self.cell_size * 0.5;
                    let wz = cz as f32 * self.cell_size + self.offset_z + self.cell_size * 0.5;
                    best = Some((wx, wz, q));
                }
            }
        }

        best
    }
}
