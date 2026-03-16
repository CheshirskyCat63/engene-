use crate::world::heightmap::Heightmap;
use pathfinding::directed::astar::astar;

pub struct NavGrid {
    width: usize,
    height: usize,
    spacing: f32,
    passable: Vec<bool>,
}

impl NavGrid {
    pub fn from_heightmap(heightmap: &Heightmap, spacing: f32, max_slope: f32) -> Self {
        let ws = heightmap.world_size;
        let w = (ws / spacing) as usize;
        let h = w;
        let mut passable = vec![true; w * h];

        for gz in 0..h {
            for gx in 0..w {
                let wx = gx as f32 * spacing;
                let wz = gz as f32 * spacing;
                let wy = heightmap.sample(wx, wz);
                let n = heightmap.normal_at(wx, wz);
                let slope = (1.0 - n[1]).abs();
                if slope > max_slope || wy < -5.0 {
                    passable[gz * w + gx] = false;
                }
            }
        }

        Self {
            width: w,
            height: h,
            spacing,
            passable,
        }
    }

    pub fn find_path(&self, start: [f32; 2], goal: [f32; 2]) -> Option<Vec<[f32; 2]>> {
        let sx = (start[0] / self.spacing).clamp(0.0, (self.width - 1) as f32) as usize;
        let sy = (start[1] / self.spacing).clamp(0.0, (self.height - 1) as f32) as usize;
        let gx = (goal[0] / self.spacing).clamp(0.0, (self.width - 1) as f32) as usize;
        let gy = (goal[1] / self.spacing).clamp(0.0, (self.height - 1) as f32) as usize;

        let start_idx = sy * self.width + sx;
        let goal_idx = gy * self.width + gx;

        if !self.passable[start_idx] || !self.passable[goal_idx] {
            return None;
        }

        let result = astar(
            &start_idx,
            |&idx| {
                let x = idx % self.width;
                let y = idx / self.width;
                let mut neighbors = Vec::with_capacity(8);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                            continue;
                        }
                        let ni = ny as usize * self.width + nx as usize;
                        if self.passable[ni] {
                            let cost = if dx.abs() + dy.abs() == 2 { 14u32 } else { 10u32 };
                            neighbors.push((ni, cost));
                        }
                    }
                }
                neighbors
            },
            |&idx| {
                let x = idx % self.width;
                let y = idx / self.width;
                let dx = (x as i32 - gx as i32).unsigned_abs();
                let dy = (y as i32 - gy as i32).unsigned_abs();
                (dx.max(dy) * 10 + dx.min(dy) * 4) as u32
            },
            |&idx| idx == goal_idx,
        );

        result.map(|(path, _)| {
            path.into_iter()
                .map(|idx| {
                    let x = (idx % self.width) as f32 * self.spacing;
                    let z = (idx / self.width) as f32 * self.spacing;
                    [x, z]
                })
                .collect()
        })
    }
}
