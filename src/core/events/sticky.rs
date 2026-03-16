use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct StickyEvents {
    events: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl StickyEvents {
    pub fn new() -> Self {
        Self { events: HashMap::new() }
    }

    pub fn set<E: 'static + Send + Sync>(&mut self, event: E) {
        self.events.insert(TypeId::of::<E>(), Box::new(event));
    }

    pub fn get<E: 'static>(&self) -> Option<&E> {
        self.events.get(&TypeId::of::<E>()).and_then(|e| e.downcast_ref::<E>())
    }

    pub fn remove<E: 'static>(&mut self) -> bool {
        self.events.remove(&TypeId::of::<E>()).is_some()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for StickyEvents {
    fn default() -> Self { Self::new() }
}
