use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TerrainMaskPatch {
    pub wetness: f32,
    pub scorched: f32,
    pub disturbed: f32,
}

impl Default for TerrainMaskPatch {
    fn default() -> Self {
        Self {
            wetness: 0.0,
            scorched: 0.0,
            disturbed: 0.0,
        }
    }
}

pub struct TerrainMaskStore {
    patches: HashMap<(i32, i32), TerrainMaskPatch>,
}

impl TerrainMaskStore {
    pub fn new() -> Self {
        Self {
            patches: HashMap::new(),
        }
    }

    pub fn apply_scorch(&mut self, cx: i32, cz: i32, intensity: f32) {
        let patch = self.patches.entry((cx, cz)).or_default();
        patch.scorched = (patch.scorched + intensity).min(1.0);
    }

    pub fn apply_wetness(&mut self, cx: i32, cz: i32, amount: f32) {
        let patch = self.patches.entry((cx, cz)).or_default();
        patch.wetness = (patch.wetness + amount).min(1.0);
    }

    pub fn apply_disturbance(&mut self, cx: i32, cz: i32, amount: f32) {
        let patch = self.patches.entry((cx, cz)).or_default();
        patch.disturbed = (patch.disturbed + amount).min(1.0);
    }

    pub fn update(&mut self, dt: f32) {
        for patch in self.patches.values_mut() {
            patch.wetness = (patch.wetness - dt * 0.005).max(0.0);
        }
        self.patches.retain(|_, p| p.wetness > 0.001 || p.scorched > 0.001 || p.disturbed > 0.001);
    }

    pub fn get(&self, cx: i32, cz: i32) -> Option<&TerrainMaskPatch> {
        self.patches.get(&(cx, cz))
    }
}
