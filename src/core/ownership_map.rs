use std::any::TypeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessKind {
    Owner,
    Reader,
    Writer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationTiming {
    Startup,
    PreTick,
    FixedTick,
    PostTick,
    Shutdown,
    Async,
}

#[derive(Clone, Debug)]
pub struct ResourceOwnership {
    pub resource_name: String,
    pub type_id: TypeId,
    pub owner_system: String,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    pub mutation_timing: Vec<MutationTiming>,
    pub thread_safety: ThreadSafety,
    pub notes: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadSafety {
    MainThreadOnly,
    AnyThread,
    ReadParallelWriteExclusive,
}

#[derive(Clone, Debug)]
pub struct EventOwnership {
    pub event_name: String,
    pub type_id: TypeId,
    pub publishers: Vec<String>,
    pub subscribers: Vec<String>,
    pub bus: String,
    pub deterministic: bool,
}

pub struct OwnershipMap {
    resources: Vec<ResourceOwnership>,
    events: Vec<EventOwnership>,
}

impl OwnershipMap {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn register_resource(&mut self, ownership: ResourceOwnership) {
        self.resources.push(ownership);
    }

    pub fn register_event(&mut self, ownership: EventOwnership) {
        self.events.push(ownership);
    }

    pub fn resources_owned_by(&self, system: &str) -> Vec<&ResourceOwnership> {
        self.resources.iter()
            .filter(|r| r.owner_system == system)
            .collect()
    }

    pub fn systems_reading(&self, resource_name: &str) -> Vec<&str> {
        self.resources.iter()
            .filter(|r| r.resource_name == resource_name)
            .flat_map(|r| r.readers.iter().map(|s| s.as_str()))
            .collect()
    }

    pub fn systems_writing(&self, resource_name: &str) -> Vec<&str> {
        self.resources.iter()
            .filter(|r| r.resource_name == resource_name)
            .flat_map(|r| r.writers.iter().map(|s| s.as_str()))
            .collect()
    }

    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        for res in &self.resources {
            if res.owner_system.is_empty() {
                issues.push(format!("Resource '{}' has no owner", res.resource_name));
            }
            for writer in &res.writers {
                if *writer != res.owner_system && res.thread_safety == ThreadSafety::MainThreadOnly {
                    issues.push(format!(
                        "Resource '{}' is main-thread-only but '{}' writes to it",
                        res.resource_name, writer
                    ));
                }
            }
        }

        for evt in &self.events {
            if evt.publishers.is_empty() {
                issues.push(format!("Event '{}' has no publishers", evt.event_name));
            }
            if evt.subscribers.is_empty() {
                issues.push(format!("Event '{}' has no subscribers", evt.event_name));
            }
        }

        issues
    }

    pub fn resource_count(&self) -> usize { self.resources.len() }
    pub fn event_count(&self) -> usize { self.events.len() }

    pub fn resources(&self) -> &[ResourceOwnership] { &self.resources }
    pub fn events(&self) -> &[EventOwnership] { &self.events }
}

impl Default for OwnershipMap {
    fn default() -> Self { Self::new() }
}
