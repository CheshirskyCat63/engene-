use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    frozen: bool,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            frozen: false,
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        assert!(
            !self.frozen,
            "Resources are frozen after build() -- cannot insert {} at runtime",
            std::any::type_name::<T>()
        );
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T>())
    }

    /// Temporarily remove a resource. Use `insert_runtime()` to put it back.
    pub fn take<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.map.remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast::<T>().ok())
            .map(|b| *b)
    }

    /// Insert a resource even if frozen (for re-inserting after `take()`).
    pub fn insert_runtime<T: 'static + Send + Sync>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    pub fn contains_raw(&self, type_id: TypeId) -> bool {
        self.map.contains_key(&type_id)
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }

    /// Returns TypeIds of all inserted resources (for doctor orphan detection).
    pub fn type_ids(&self) -> Vec<TypeId> {
        self.map.keys().copied().collect()
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
