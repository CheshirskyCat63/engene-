use std::collections::HashMap;
use std::any::Any;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugCategory {
    World,
    AI,
    Physics,
    Graphics,
    Budgets,
}

pub struct DebugView {
    pub name: String,
    pub category: DebugCategory,
    pub enabled: bool,
}

pub struct DebugRegistry {
    views: Vec<DebugView>,
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl DebugRegistry {
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            data: HashMap::new(),
        }
    }

    pub fn register_view(&mut self, name: &str, category: DebugCategory) {
        self.views.push(DebugView {
            name: name.to_string(),
            category,
            enabled: false,
        });
    }

    pub fn toggle_view(&mut self, name: &str) {
        if let Some(view) = self.views.iter_mut().find(|v| v.name == name) {
            view.enabled = !view.enabled;
        }
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.views.iter().find(|v| v.name == name).map_or(false, |v| v.enabled)
    }

    pub fn set_data<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get_data<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|d| d.downcast_ref::<T>())
    }

    pub fn views(&self) -> &[DebugView] {
        &self.views
    }

    pub fn views_in_category(&self, category: DebugCategory) -> Vec<&DebugView> {
        self.views.iter().filter(|v| v.category == category).collect()
    }
}

impl Default for DebugRegistry {
    fn default() -> Self { Self::new() }
}
