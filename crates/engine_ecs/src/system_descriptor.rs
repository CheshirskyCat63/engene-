use std::any::TypeId;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum DeterminismTier {
    Hard,
    Soft,
    NonDeterministic,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum LowSpecPolicy {
    Off,
    ReducedCadence,
    SimplifiedPath,
    NeverCut,
}

impl Default for LowSpecPolicy {
    fn default() -> Self {
        Self::SimplifiedPath
    }
}

#[derive(Clone, Debug, Default)]
pub struct SystemOrdering {
    pub before: Vec<&'static str>,
    pub after: Vec<&'static str>,
    pub requires: Vec<&'static str>,
    pub conflicts_with: Vec<&'static str>,
}

#[derive(Clone, Debug)]
pub struct SystemDescriptor {
    pub name: &'static str,
    pub reads_components: Vec<TypeId>,
    pub writes_components: Vec<TypeId>,
    pub reads_resources: Vec<TypeId>,
    pub writes_resources: Vec<TypeId>,
    pub emits_events: Vec<TypeId>,
    pub reads_events: Vec<TypeId>,
    pub parallel_safe: bool,
    pub determinism: DeterminismTier,
    pub headless_compatible: bool,
    pub ordering: SystemOrdering,
    pub low_spec_policy: LowSpecPolicy,
}

impl SystemDescriptor {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            reads_components: Vec::new(),
            writes_components: Vec::new(),
            reads_resources: Vec::new(),
            writes_resources: Vec::new(),
            emits_events: Vec::new(),
            reads_events: Vec::new(),
            parallel_safe: false,
            determinism: DeterminismTier::Hard,
            headless_compatible: true,
            ordering: SystemOrdering::default(),
            low_spec_policy: LowSpecPolicy::default(),
        }
    }

    pub fn reads_component<T: 'static>(mut self) -> Self {
        self.reads_components.push(TypeId::of::<T>());
        self
    }

    pub fn writes_component<T: 'static>(mut self) -> Self {
        self.writes_components.push(TypeId::of::<T>());
        self
    }

    pub fn reads_resource<T: 'static>(mut self) -> Self {
        self.reads_resources.push(TypeId::of::<T>());
        self
    }

    pub fn writes_resource<T: 'static>(mut self) -> Self {
        self.writes_resources.push(TypeId::of::<T>());
        self
    }

    pub fn emits_event<T: 'static>(mut self) -> Self {
        self.emits_events.push(TypeId::of::<T>());
        self
    }

    pub fn reads_event<T: 'static>(mut self) -> Self {
        self.reads_events.push(TypeId::of::<T>());
        self
    }

    pub fn with_parallel(mut self, safe: bool) -> Self {
        self.parallel_safe = safe;
        self
    }

    pub fn with_determinism(mut self, tier: DeterminismTier) -> Self {
        self.determinism = tier;
        self
    }

    pub fn with_headless(mut self, compatible: bool) -> Self {
        self.headless_compatible = compatible;
        self
    }

    pub fn before(mut self, name: &'static str) -> Self {
        self.ordering.before.push(name);
        self
    }

    pub fn after(mut self, name: &'static str) -> Self {
        self.ordering.after.push(name);
        self
    }

    pub fn requires(mut self, name: &'static str) -> Self {
        self.ordering.requires.push(name);
        self
    }

    pub fn with_low_spec_policy(mut self, policy: LowSpecPolicy) -> Self {
        self.low_spec_policy = policy;
        self
    }
}
