use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::components::SimulationLevel;

const IGNITION_TEMP: f32 = 100.0;
const AMBIENT_TEMP: f32 = 20.0;
const BURN_RATE: f32 = 0.15;
const HEAT_OUTPUT: f32 = 200.0;
const DIFFUSION_RATE: f32 = 0.08;
const COOLDOWN_RATE: f32 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FireState {
    Unlit,
    Burning,
    BurnedOut,
}

#[derive(Clone, Debug)]
pub struct FireCell {
    pub state: FireState,
    pub fuel: f32,
    pub temperature: f32,
    pub flammability: f32,
}

impl Default for FireCell {
    fn default() -> Self {
        Self {
            state: FireState::Unlit,
            fuel: 0.0,
            temperature: AMBIENT_TEMP,
            flammability: 0.0,
        }
    }
}

pub struct FireGrid {
    width: u32,
    height: u32,
    cells: Vec<FireCell>,
    scratch: Vec<f32>,
    pub wind: [f32; 2],
    tick_counter: u64,
}

impl FireGrid {
    pub fn new() -> Self {
        let w = GRID_SIZE;
        let h = GRID_SIZE;
        let n = (w * h) as usize;
        Self {
            width: w,
            height: h,
            cells: vec![FireCell::default(); n],
            scratch: vec![0.0; n],
            wind: [0.0, 0.0],
            tick_counter: 0,
        }
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> &FireCell {
        &self.cells[self.idx(x, y)]
    }

    pub fn set_fuel(&mut self, x: u32, y: u32, fuel: f32, flammability: f32) {
        let i = self.idx(x, y);
        self.cells[i].fuel = fuel;
        self.cells[i].flammability = flammability;
    }

    pub fn ignite(&mut self, x: u32, y: u32) {
        let i = self.idx(x, y);
        if self.cells[i].fuel > 0.0 && self.cells[i].state == FireState::Unlit {
            self.cells[i].temperature = IGNITION_TEMP + 50.0;
            self.cells[i].state = FireState::Burning;
        }
    }

    pub fn extinguish(&mut self, x: u32, y: u32) {
        let i = self.idx(x, y);
        self.cells[i].temperature = AMBIENT_TEMP;
        if self.cells[i].state == FireState::Burning {
            self.cells[i].state = FireState::Unlit;
        }
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

        for i in 0..self.scratch.len() {
            self.scratch[i] = 0.0;
        }

        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let cell = &self.cells[i];

                if cell.state != FireState::Burning {
                    continue;
                }

                let heat = HEAT_OUTPUT * dt;
                let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

                for &(dx, dy) in &neighbors {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = self.idx(nx as u32, ny as u32);

                    let wind_dot = dx as f32 * self.wind[0] + dy as f32 * self.wind[1];
                    let wind_bonus = (1.0 + wind_dot.max(0.0) * 2.0).min(3.0);

                    self.scratch[ni] += heat * DIFFUSION_RATE * wind_bonus;
                }
            }
        }

        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let cell = &mut self.cells[i];

                cell.temperature += self.scratch[i];

                match cell.state {
                    FireState::Unlit => {
                        if cell.fuel > 0.0
                            && cell.flammability > 0.0
                            && cell.temperature >= IGNITION_TEMP * (1.0 - cell.flammability * 0.5)
                        {
                            cell.state = FireState::Burning;
                        }
                        cell.temperature =
                            (cell.temperature - COOLDOWN_RATE * dt).max(AMBIENT_TEMP);
                    }
                    FireState::Burning => {
                        cell.fuel -= BURN_RATE * cell.flammability * dt;
                        cell.temperature =
                            (IGNITION_TEMP + HEAT_OUTPUT * cell.flammability).max(cell.temperature);
                        if cell.fuel <= 0.0 {
                            cell.fuel = 0.0;
                            cell.state = FireState::BurnedOut;
                        }
                    }
                    FireState::BurnedOut => {
                        cell.temperature =
                            (cell.temperature - COOLDOWN_RATE * 2.0 * dt).max(AMBIENT_TEMP);
                    }
                }
            }
        }
    }

    fn update_statistical(&mut self, dt: f32) {
        let w = self.width;
        let h = self.height;

        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                let cell = &mut self.cells[i];

                if cell.state == FireState::Burning {
                    cell.fuel -= BURN_RATE * cell.flammability * dt;
                    if cell.fuel <= 0.0 {
                        cell.fuel = 0.0;
                        cell.state = FireState::BurnedOut;
                    }

                    let spread_chance = cell.flammability * 0.1 * dt;
                    let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    for &(dx, dy) in &neighbors {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let ni = self.idx(nx as u32, ny as u32);
                        let neighbor = &self.cells[ni];
                        if neighbor.state == FireState::Unlit
                            && neighbor.fuel > 0.0
                            && neighbor.flammability > 0.0
                        {
                            let roll = (x.wrapping_mul(73)
                                ^ y.wrapping_mul(179)
                                ^ self.tick_counter as u32)
                                as f32
                                / u32::MAX as f32;
                            if roll < spread_chance * neighbor.flammability {
                                let nc = &mut self.cells[ni];
                                nc.state = FireState::Burning;
                                nc.temperature = IGNITION_TEMP;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn burning_cells(&self) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[self.idx(x, y)].state == FireState::Burning {
                    result.push((x, y));
                }
            }
        }
        result
    }

    pub fn cell_world_pos(x: u32, y: u32) -> (f32, f32) {
        (
            x as f32 * CELL_SIZE + CELL_SIZE * 0.5,
            y as f32 * CELL_SIZE + CELL_SIZE * 0.5,
        )
    }

    pub fn is_near_fire(&self, world_x: f32, world_y: f32, radius: f32) -> bool {
        let cx = (world_x / CELL_SIZE) as i32;
        let cy = (world_y / CELL_SIZE) as i32;
        let cr = (radius / CELL_SIZE).ceil() as i32 + 1;

        for dy in -cr..=cr {
            for dx in -cr..=cr {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
                    continue;
                }
                if self.cells[self.idx(nx as u32, ny as u32)].state == FireState::Burning {
                    let (fx, fy) = Self::cell_world_pos(nx as u32, ny as u32);
                    let dist2 = (fx - world_x).powi(2) + (fy - world_y).powi(2);
                    if dist2 < radius * radius {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn active_fire_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.state == FireState::Burning)
            .count()
    }
}
