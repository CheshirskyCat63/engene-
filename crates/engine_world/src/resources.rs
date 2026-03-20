use crate::world::biome::Biome;
use crate::world::cell::GRID_SIZE;

pub struct ResourceGrid {
    pub food: Vec<f32>,
    pub water: Vec<f32>,
    pub carcasses: Vec<Carcass>,
}

#[derive(Clone, Debug)]
pub struct Carcass {
    pub x: f32,
    pub y: f32,
    pub cell_x: u32,
    pub cell_y: u32,
    pub food_remaining: f32,
    pub decay_timer: f32,
    pub source_name: String,
}

impl ResourceGrid {
    pub fn new(biomes: &[Biome]) -> Self {
        let n = (GRID_SIZE * GRID_SIZE) as usize;
        let mut food = vec![0.0f32; n];
        let mut water = vec![0.0f32; n];
        for (i, b) in biomes.iter().enumerate() {
            food[i] = b.food_density();
            water[i] = b.water_density();
        }
        Self {
            food,
            water,
            carcasses: Vec::new(),
        }
    }

    pub fn idx(cx: u32, cy: u32) -> usize {
        (cy * GRID_SIZE + cx) as usize
    }

    pub fn food_at(&self, cx: u32, cy: u32) -> f32 {
        self.food[Self::idx(cx, cy)]
    }

    pub fn water_at(&self, cx: u32, cy: u32) -> f32 {
        self.water[Self::idx(cx, cy)]
    }

    pub fn consume_food(&mut self, cx: u32, cy: u32, amount: f32) -> f32 {
        let i = Self::idx(cx, cy);
        let taken = amount.min(self.food[i]);
        self.food[i] -= taken;
        taken
    }

    pub fn consume_water(&mut self, cx: u32, cy: u32, amount: f32) -> f32 {
        let i = Self::idx(cx, cy);
        let taken = amount.min(self.water[i]);
        self.water[i] -= taken;
        taken
    }

    pub fn add_carcass(&mut self, x: f32, y: f32, cx: u32, cy: u32, food: f32, name: String) {
        self.carcasses.push(Carcass {
            x,
            y,
            cell_x: cx,
            cell_y: cy,
            food_remaining: food,
            decay_timer: 600.0,
            source_name: name,
        });
        self.food[Self::idx(cx, cy)] += food * 0.3;
    }

    pub fn consume_carcass_near(&mut self, x: f32, y: f32, amount: f32) -> f32 {
        let r2 = 40.0 * 40.0;
        let mut total_eaten = 0.0;
        for c in &mut self.carcasses {
            if (c.x - x).powi(2) + (c.y - y).powi(2) < r2 {
                let taken = amount.min(c.food_remaining);
                c.food_remaining -= taken;
                total_eaten += taken;
                if total_eaten >= amount {
                    break;
                }
            }
        }
        total_eaten
    }

    pub fn find_nearest_carcass(&self, x: f32, y: f32, radius: f32) -> Option<(f32, f32)> {
        let r2 = radius * radius;
        self.carcasses
            .iter()
            .filter(|c| c.food_remaining > 0.05)
            .filter(|c| (c.x - x).powi(2) + (c.y - y).powi(2) < r2)
            .min_by(|a, b| {
                let da = (a.x - x).powi(2) + (a.y - y).powi(2);
                let db = (b.x - x).powi(2) + (b.y - y).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .map(|c| (c.x, c.y))
    }

    pub fn tick(&mut self, delta: f32, biomes: &[Biome]) {
        for (i, b) in biomes.iter().enumerate() {
            let cap = b.food_density();
            self.food[i] = (self.food[i] + delta * 0.003 * cap).min(cap);
            let wcap = b.water_density();
            self.water[i] = (self.water[i] + delta * 0.01 * wcap).min(wcap);
        }
        for c in &mut self.carcasses {
            c.decay_timer -= delta;
            c.food_remaining = (c.food_remaining - delta * 0.001).max(0.0);
        }
        self.carcasses
            .retain(|c| c.decay_timer > 0.0 && c.food_remaining > 0.01);
    }

    pub fn best_food_cell(
        &self,
        near_cx: u32,
        near_cy: u32,
        search_radius: u32,
    ) -> Option<(u32, u32)> {
        let mut best: Option<(u32, u32, f32)> = None;
        let r = search_radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = near_cx as i32 + dx;
                let ny = near_cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= GRID_SIZE as i32 || ny >= GRID_SIZE as i32 {
                    continue;
                }
                let (ux, uy) = (nx as u32, ny as u32);
                let f = self.food_at(ux, uy);
                if best.map_or(true, |(_, _, bf)| f > bf) && f > 0.05 {
                    best = Some((ux, uy, f));
                }
            }
        }
        best.map(|(x, y, _)| (x, y))
    }

    pub fn best_water_cell(
        &self,
        near_cx: u32,
        near_cy: u32,
        search_radius: u32,
    ) -> Option<(u32, u32)> {
        let mut best: Option<(u32, u32, f32)> = None;
        let r = search_radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = near_cx as i32 + dx;
                let ny = near_cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= GRID_SIZE as i32 || ny >= GRID_SIZE as i32 {
                    continue;
                }
                let (ux, uy) = (nx as u32, ny as u32);
                let w = self.water_at(ux, uy);
                if best.map_or(true, |(_, _, bw)| w > bw) && w > 0.05 {
                    best = Some((ux, uy, w));
                }
            }
        }
        best.map(|(x, y, _)| (x, y))
    }
}
