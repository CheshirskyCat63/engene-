use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::components::SimulationLevel;

const FLOW_RATE: f32 = 0.25;
const EVAPORATION: f32 = 0.001;
const MIN_WATER: f32 = 0.001;

#[derive(Clone, Debug)]
pub struct WaterCell {
    pub height: f32,
    pub water_level: f32,
    pub flow: [f32; 4],
    pub is_container: bool,
}

impl Default for WaterCell {
    fn default() -> Self {
        Self {
            height: 0.0,
            water_level: 0.0,
            flow: [0.0; 4],
            is_container: false,
        }
    }
}

pub struct WaterGrid {
    width: u32,
    height: u32,
    cells: Vec<WaterCell>,
    pub rain_intensity: f32,
    tick_counter: u64,
}

impl WaterGrid {
    pub fn new() -> Self {
        let w = GRID_SIZE;
        let h = GRID_SIZE;
        Self {
            width: w,
            height: h,
            cells: vec![WaterCell::default(); (w * h) as usize],
            rain_intensity: 0.0,
            tick_counter: 0,
        }
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> &WaterCell {
        &self.cells[self.idx(x, y)]
    }

    pub fn set_terrain_height(&mut self, x: u32, y: u32, h: f32) {
        let i = self.idx(x, y);
        self.cells[i].height = h;
    }

    pub fn mark_container(&mut self, x: u32, y: u32) {
        let i = self.idx(x, y);
        self.cells[i].is_container = true;
    }

    pub fn add_water(&mut self, x: u32, y: u32, amount: f32) {
        let i = self.idx(x, y);
        self.cells[i].water_level += amount;
    }

    pub fn water_level_at_world(&self, wx: f32, wz: f32) -> f32 {
        let cx = ((wx / CELL_SIZE) as u32).min(self.width - 1);
        let cy = ((wz / CELL_SIZE) as u32).min(self.height - 1);
        self.cells[self.idx(cx, cy)].water_level
    }

    pub fn update(&mut self, dt: f32, sim_level: SimulationLevel) {
        self.tick_counter += 1;

        match sim_level {
            SimulationLevel::L0 => self.update_full(dt),
            SimulationLevel::L1 => {
                if self.tick_counter % 4 == 0 {
                    self.update_full(dt * 4.0);
                }
            }
            SimulationLevel::L2 => {
                if self.tick_counter % 16 == 0 {
                    self.update_statistical(dt * 16.0);
                }
            }
            SimulationLevel::L3 => {}
        }
    }

    fn update_full(&mut self, dt: f32) {
        let w = self.width;
        let h = self.height;

        if self.rain_intensity > 0.0 {
            let rain_per_cell = self.rain_intensity * dt * 0.01;
            for cell in &mut self.cells {
                cell.water_level += rain_per_cell;
            }
        }

        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let total_h = self.cells[i].height + self.cells[i].water_level;

                let neighbors: [(i32, i32, usize); 4] =
                    [(-1, 0, 0), (1, 0, 1), (0, -1, 2), (0, 1, 3)];

                for &(dx, dy, dir) in &neighbors {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        self.cells[i].flow[dir] = 0.0;
                        continue;
                    }
                    let ni = self.idx(nx as u32, ny as u32);
                    let n_total_h = self.cells[ni].height + self.cells[ni].water_level;

                    let diff = total_h - n_total_h;
                    if diff > 0.0 && !self.cells[i].is_container {
                        self.cells[i].flow[dir] =
                            (self.cells[i].flow[dir] + diff * FLOW_RATE * dt).max(0.0);
                    } else {
                        self.cells[i].flow[dir] = (self.cells[i].flow[dir] - 0.1 * dt).max(0.0);
                    }
                }

                let total_out: f32 = self.cells[i].flow.iter().sum();
                if total_out > self.cells[i].water_level && total_out > 0.0 {
                    let scale = self.cells[i].water_level / total_out;
                    for f in &mut self.cells[i].flow {
                        *f *= scale;
                    }
                }
            }
        }

        let mut deltas = vec![0.0f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let out: f32 = self.cells[i].flow.iter().sum();
                deltas[i] -= out;

                let neighbors: [(i32, i32, usize); 4] =
                    [(-1, 0, 1), (1, 0, 0), (0, -1, 3), (0, 1, 2)];
                for &(dx, dy, opp_dir) in &neighbors {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = self.idx(nx as u32, ny as u32);
                    deltas[i] += self.cells[ni].flow[opp_dir];
                }
            }
        }

        for (i, cell) in self.cells.iter_mut().enumerate() {
            cell.water_level = (cell.water_level + deltas[i]).max(0.0);
            if !cell.is_container {
                cell.water_level = (cell.water_level - EVAPORATION * dt).max(0.0);
            }
            if cell.water_level < MIN_WATER {
                cell.water_level = 0.0;
            }
        }
    }

    fn update_statistical(&mut self, dt: f32) {
        if self.rain_intensity > 0.0 {
            let rain = self.rain_intensity * dt * 0.01;
            for cell in &mut self.cells {
                if cell.is_container {
                    cell.water_level += rain;
                } else {
                    cell.water_level = (cell.water_level + rain * 0.1 - EVAPORATION * dt).max(0.0);
                }
            }
        }
    }

    pub fn buoyancy_force(&self, wx: f32, wz: f32, obj_bottom_y: f32) -> f32 {
        let water = self.water_level_at_world(wx, wz);
        if water <= 0.0 {
            return 0.0;
        }
        let cx = ((wx / CELL_SIZE) as u32).min(self.width - 1);
        let cy = ((wz / CELL_SIZE) as u32).min(self.height - 1);
        let surface_y = self.cells[self.idx(cx, cy)].height + water;
        let submerged = (surface_y - obj_bottom_y).max(0.0);
        submerged * 9.81 * 1000.0
    }
}
