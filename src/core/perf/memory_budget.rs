use std::collections::HashMap;

pub struct MemoryBudgetEntry {
    pub name: &'static str,
    pub budget_bytes: usize,
    pub used_bytes: usize,
}

pub struct MemoryBudgetRegistry {
    entries: HashMap<&'static str, MemoryBudgetEntry>,
}

impl MemoryBudgetRegistry {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn register(&mut self, name: &'static str, budget_bytes: usize) {
        self.entries.insert(name, MemoryBudgetEntry {
            name,
            budget_bytes,
            used_bytes: 0,
        });
    }

    pub fn record_usage(&mut self, name: &str, used_bytes: usize) {
        if let Some(entry) = self.entries.get_mut(name) {
            entry.used_bytes = used_bytes;
        }
    }

    pub fn is_over_budget(&self, name: &str) -> bool {
        self.entries.get(name).map_or(false, |e| e.used_bytes > e.budget_bytes)
    }

    pub fn total_budget(&self) -> usize {
        self.entries.values().map(|e| e.budget_bytes).sum()
    }

    pub fn total_used(&self) -> usize {
        self.entries.values().map(|e| e.used_bytes).sum()
    }

    pub fn entries(&self) -> impl Iterator<Item = &MemoryBudgetEntry> {
        self.entries.values()
    }
}

impl Default for MemoryBudgetRegistry {
    fn default() -> Self { Self::new() }
}
