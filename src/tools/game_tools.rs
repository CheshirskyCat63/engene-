use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EncounterDescriptor {
    pub name: String,
    pub faction: String,
    pub spawn_prefabs: Vec<SpawnEntry>,
    pub trigger_radius: f32,
    pub difficulty: f32,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SpawnEntry {
    pub prefab_name: String,
    pub count: u32,
    pub position_offset: [f32; 3],
}

pub struct EncounterEditor {
    encounters: Vec<EncounterDescriptor>,
    selected: Option<usize>,
}

impl EncounterEditor {
    pub fn new() -> Self {
        Self { encounters: Vec::new(), selected: None }
    }

    pub fn add_encounter(&mut self, encounter: EncounterDescriptor) {
        self.encounters.push(encounter);
    }

    pub fn remove_encounter(&mut self, index: usize) {
        if index < self.encounters.len() {
            self.encounters.remove(index);
            self.selected = None;
        }
    }

    pub fn encounters(&self) -> &[EncounterDescriptor] {
        &self.encounters
    }

    pub fn selected(&self) -> Option<&EncounterDescriptor> {
        self.selected.and_then(|i| self.encounters.get(i))
    }

    pub fn count(&self) -> usize {
        self.encounters.len()
    }
}

impl Default for EncounterEditor {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct FactionData {
    pub name: String,
    pub relations: HashMap<String, f32>,
    pub color: [f32; 4],
}

pub struct FactionEditor {
    factions: Vec<FactionData>,
}

impl FactionEditor {
    pub fn new() -> Self {
        Self { factions: Vec::new() }
    }

    pub fn add_faction(&mut self, faction: FactionData) {
        self.factions.push(faction);
    }

    pub fn set_relation(&mut self, a: &str, b: &str, value: f32) {
        for faction in &mut self.factions {
            if faction.name == a {
                faction.relations.insert(b.to_string(), value);
            }
            if faction.name == b {
                faction.relations.insert(a.to_string(), value);
            }
        }
    }

    pub fn factions(&self) -> &[FactionData] {
        &self.factions
    }

    pub fn count(&self) -> usize {
        self.factions.len()
    }
}

impl Default for FactionEditor {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct SpawnSetDescriptor {
    pub name: String,
    pub entries: Vec<SpawnSetEntry>,
    pub budget_cost: f32,
}

#[derive(Clone, Debug)]
pub struct SpawnSetEntry {
    pub prefab_name: String,
    pub weight: f32,
    pub min_count: u32,
    pub max_count: u32,
}

pub struct SpawnSetEditor {
    sets: Vec<SpawnSetDescriptor>,
}

impl SpawnSetEditor {
    pub fn new() -> Self {
        Self { sets: Vec::new() }
    }

    pub fn add_set(&mut self, set: SpawnSetDescriptor) {
        self.sets.push(set);
    }

    pub fn sets(&self) -> &[SpawnSetDescriptor] {
        &self.sets
    }

    pub fn count(&self) -> usize {
        self.sets.len()
    }
}

impl Default for SpawnSetEditor {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct EconomyBalanceEntry {
    pub resource_name: String,
    pub base_value: f32,
    pub production_rate: f32,
    pub consumption_rate: f32,
    pub max_stockpile: f32,
}

pub struct EconomyBalanceTool {
    entries: Vec<EconomyBalanceEntry>,
}

impl EconomyBalanceTool {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: EconomyBalanceEntry) {
        self.entries.push(entry);
    }

    pub fn equilibrium_time(&self, resource: &str) -> Option<f32> {
        let entry = self.entries.iter().find(|e| e.resource_name == resource)?;
        let net_rate = entry.production_rate - entry.consumption_rate;
        if net_rate.abs() < 0.001 { return None; }
        Some(entry.max_stockpile / net_rate.abs())
    }

    pub fn entries(&self) -> &[EconomyBalanceEntry] {
        &self.entries
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for EconomyBalanceTool {
    fn default() -> Self { Self::new() }
}
