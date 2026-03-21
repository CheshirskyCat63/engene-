use std::any::TypeId;

#[derive(Clone, Debug)]
pub struct AccessDescriptor {
    pub reads_components: Vec<TypeId>,
    pub writes_components: Vec<TypeId>,
    pub reads_resources: Vec<TypeId>,
    pub writes_resources: Vec<TypeId>,
    pub emits_events: Vec<TypeId>,
    pub reads_events: Vec<TypeId>,
}

impl AccessDescriptor {
    pub fn new() -> Self {
        Self {
            reads_components: Vec::new(),
            writes_components: Vec::new(),
            reads_resources: Vec::new(),
            writes_resources: Vec::new(),
            emits_events: Vec::new(),
            reads_events: Vec::new(),
        }
    }

    pub fn has_write_conflict(&self, other: &AccessDescriptor) -> bool {
        for w in &self.writes_components {
            if other.writes_components.contains(w) || other.reads_components.contains(w) {
                return true;
            }
        }
        for w in &other.writes_components {
            if self.reads_components.contains(w) {
                return true;
            }
        }
        for w in &self.writes_resources {
            if other.writes_resources.contains(w) || other.reads_resources.contains(w) {
                return true;
            }
        }
        for w in &other.writes_resources {
            if self.reads_resources.contains(w) {
                return true;
            }
        }
        false
    }
}

impl Default for AccessDescriptor {
    fn default() -> Self {
        Self::new()
    }
}
