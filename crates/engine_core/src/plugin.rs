use std::any::TypeId;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum TickFreq {
    EveryFrame,
    EveryN(u32),
    OnEvent,
}

// Temporary stubs for cross-module dependencies
pub struct Ecs;
pub struct EventBus;
pub trait EngineSystem {
    fn name(&self) -> &str;
    fn descriptor(&self) -> SystemDescriptor;
}

#[derive(Clone, Debug)]
pub struct DeterminismAuditEntry {
    pub name: &'static str,
    pub parallel_safe: bool,
    pub reads_components: Vec<std::any::TypeId>,
    pub writes_components: Vec<std::any::TypeId>,
    pub reads_resources: Vec<std::any::TypeId>,
}

use crate::registry::Resources;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum DeterminismTier {
    Pure,
    Deterministic,
    Safe,
    Hard,
    Soft,
    NonDeterministic,
}

pub struct SystemDescriptor {
    pub name: &'static str,
    pub parallel_safe: bool,
    pub reads_components: Vec<std::any::TypeId>,
    pub writes_components: Vec<std::any::TypeId>,
    pub reads_resources: Vec<std::any::TypeId>,
    pub writes_resources: Vec<std::any::TypeId>,
    pub ordering: SystemOrdering,
    pub determinism: DeterminismTier,
    pub headless_compatible: bool,
}

pub struct SystemOrdering {
    pub before: Vec<&'static str>,
    pub after: Vec<&'static str>,
    pub requires: Vec<&'static str>,
}

pub struct SystemMeta {
    pub name: String,
    pub tick_frequency: TickFreq,
}

pub trait Plugin {
    fn name(&self) -> &str;

    fn dependencies(&self) -> Vec<TypeId> {
        vec![]
    }

    fn build(&self, builder: &mut EngineBuilder);
}

pub struct EngineBuilder {
    pub systems: Vec<(Box<dyn EngineSystem>, SystemMeta)>,
    pub resources: Resources,
    pub init_fns: Vec<Box<dyn FnOnce(&mut Ecs, &mut Resources)>>,
    installed_plugins: HashSet<String>,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            resources: Resources::new(),
            init_fns: Vec::new(),
            installed_plugins: HashSet::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn EngineSystem>, meta: SystemMeta) {
        println!("[builder] registered system: {}", meta.name);
        self.systems.push((system, meta));
    }

    pub fn add_system_default(&mut self, system: Box<dyn EngineSystem>) {
        let name = system.name().to_string();
        let meta = SystemMeta {
            name: name.clone(),
            tick_frequency: TickFreq::EveryFrame,
        };
        self.add_system(system, meta);
    }

    pub fn add_init_fn<F: FnOnce(&mut Ecs, &mut Resources) + 'static>(&mut self, f: F) {
        self.init_fns.push(Box::new(f));
    }

    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, value: T) {
        self.resources.insert(value);
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) {
        let plugin_name = plugin.name().to_string();
        if self.installed_plugins.contains(&plugin_name) {
            println!(
                "[builder] plugin '{}' already installed, skipping",
                plugin_name
            );
            return;
        }

        for dep in plugin.dependencies() {
            assert!(
                self.resources.contains_raw(dep),
                "Plugin '{}' requires resource {:?} which has not been registered",
                plugin_name,
                dep
            );
        }

        println!("[builder] installing plugin: {}", plugin_name);
        plugin.build(self);
        self.installed_plugins.insert(plugin_name);
    }

    pub fn build(mut self, ecs: &mut Ecs) -> (Vec<Box<dyn EngineSystem>>, Resources) {
        for init_fn in self.init_fns.drain(..) {
            init_fn(ecs, &mut self.resources);
        }

        self.resources.freeze();

        let descriptors: Vec<SystemDescriptor> = self
            .systems
            .iter()
            .map(|(sys, _)| sys.descriptor())
            .collect();

        validate_dependency_graph(&descriptors);
        validate_parallel_safety(&descriptors);

        let sorted_indices = topological_sort(&descriptors);

        let mut indexed_systems: Vec<(usize, Box<dyn EngineSystem>, SystemMeta)> = self
            .systems
            .into_iter()
            .enumerate()
            .map(|(i, (sys, meta))| (i, sys, meta))
            .collect();

        let mut sorted_systems: Vec<Box<dyn EngineSystem>> =
            Vec::with_capacity(indexed_systems.len());
        for idx in sorted_indices {
            let pos = indexed_systems.iter().position(|(i, _, _)| *i == idx);
            if let Some(pos) = pos {
                let (_, sys, _) = indexed_systems.remove(pos);
                sorted_systems.push(sys);
            }
        }
        for (_, sys, _) in indexed_systems {
            sorted_systems.push(sys);
        }

        println!(
            "[builder] {} systems built, order validated",
            sorted_systems.len()
        );
        (sorted_systems, self.resources)
    }
}

fn validate_dependency_graph(descriptors: &[SystemDescriptor]) {
    let names: HashSet<&str> = descriptors.iter().map(|d| d.name).collect();

    for desc in descriptors {
        for &before in &desc.ordering.before {
            if !names.contains(before) {
                println!(
                    "[builder] warning: system '{}' declares before '{}' which doesn't exist",
                    desc.name, before
                );
            }
        }
        for &after in &desc.ordering.after {
            if !names.contains(after) {
                println!(
                    "[builder] warning: system '{}' declares after '{}' which doesn't exist",
                    desc.name, after
                );
            }
        }
        for &req in &desc.ordering.requires {
            assert!(
                names.contains(req),
                "System '{}' requires '{}' which is not registered",
                desc.name,
                req
            );
        }
    }
}

fn validate_parallel_safety(descriptors: &[SystemDescriptor]) {
    for (i, a) in descriptors.iter().enumerate() {
        for b in descriptors.iter().skip(i + 1) {
            if !a.parallel_safe || !b.parallel_safe {
                continue;
            }

            for w in &a.writes_components {
                if b.writes_components.contains(w) {
                    println!(
                        "[parallel] warning: systems '{}' and '{}' both write component {:?}",
                        a.name, b.name, w
                    );
                }
                if b.reads_components.contains(w) {
                    println!(
                        "[parallel] warning: system '{}' writes component that '{}' reads ({:?})",
                        a.name, b.name, w
                    );
                }
            }
            for w in &b.writes_components {
                if a.reads_components.contains(w) {
                    println!(
                        "[parallel] warning: system '{}' writes component that '{}' reads ({:?})",
                        b.name, a.name, w
                    );
                }
            }

            for w in &a.writes_resources {
                if b.writes_resources.contains(w) || b.reads_resources.contains(w) {
                    println!(
                        "[parallel] warning: resource conflict between '{}' and '{}' on {:?}",
                        a.name, b.name, w
                    );
                }
            }
            for w in &b.writes_resources {
                if a.reads_resources.contains(w) {
                    println!(
                        "[parallel] warning: resource conflict between '{}' and '{}' on {:?}",
                        b.name, a.name, w
                    );
                }
            }
        }
    }
}

fn topological_sort(descriptors: &[SystemDescriptor]) -> Vec<usize> {
    let name_to_idx: HashMap<&str, usize> = descriptors
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name, i))
        .collect();

    let n = descriptors.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for (i, desc) in descriptors.iter().enumerate() {
        for &before in &desc.ordering.before {
            if let Some(&j) = name_to_idx.get(before) {
                adj[i].push(j);
                in_degree[j] += 1;
            }
        }
        for &after in &desc.ordering.after {
            if let Some(&j) = name_to_idx.get(after) {
                adj[j].push(i);
                in_degree[i] += 1;
            }
        }
    }

    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result = Vec::with_capacity(n);

    while let Some(node) = queue.pop() {
        result.push(node);
        for &next in &adj[node] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push(next);
            }
        }
    }

    if result.len() != n {
        panic!(
            "[builder] FATAL: cycle detected in system dependency graph! Sorted {} of {} systems.",
            result.len(),
            n
        );
    }

    result
}
