use std::collections::HashMap;

use crate::navigation::world_graph::LocationId;

pub struct PathCache {
    cache: HashMap<(LocationId, LocationId), Vec<LocationId>>,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, from: LocationId, to: LocationId) -> Option<&[LocationId]> {
        self.cache.get(&(from, to)).map(|v| v.as_slice())
    }

    pub fn store(&mut self, from: LocationId, to: LocationId, path: Vec<LocationId>) {
        self.cache.insert((from, to), path);
    }
}
