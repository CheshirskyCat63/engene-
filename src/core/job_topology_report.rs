//! Job topology analysis for system parallelism and contention detection.

use std::any::TypeId;

use engine_core::system::EngineSystem;
use engine_core::system_descriptor::SystemDescriptor;

#[derive(Clone, Debug)]
pub enum ContentionKind {
    WriteWrite,
    ReadWrite,
    QueuePressure,
}

#[derive(Clone, Debug)]
pub struct ContentionPoint {
    pub resource: String,
    pub systems: Vec<String>,
    pub kind: ContentionKind,
}

#[derive(Clone, Debug)]
pub struct JobTopologyReport {
    pub total_jobs: usize,
    pub parallel_groups: Vec<Vec<String>>,
    pub serialized_systems: Vec<String>,
    pub fence_count: usize,
    pub contention_points: Vec<ContentionPoint>,
}

fn systems_conflict(a: &SystemDescriptor, b: &SystemDescriptor) -> Vec<(TypeId, ContentionKind)> {
    let mut conflicts = Vec::new();

    for w in &a.writes_components {
        if b.writes_components.contains(w) {
            conflicts.push((*w, ContentionKind::WriteWrite));
        } else if b.reads_components.contains(w) {
            conflicts.push((*w, ContentionKind::ReadWrite));
        }
    }
    for w in &b.writes_components {
        if a.reads_components.contains(w) && !conflicts.iter().any(|(tid, _)| *tid == *w) {
            conflicts.push((*w, ContentionKind::ReadWrite));
        }
    }

    for w in &a.writes_resources {
        if b.writes_resources.contains(w) {
            conflicts.push((*w, ContentionKind::WriteWrite));
        } else if b.reads_resources.contains(w) {
            conflicts.push((*w, ContentionKind::ReadWrite));
        }
    }
    for w in &b.writes_resources {
        if a.reads_resources.contains(w) && !conflicts.iter().any(|(tid, _)| *tid == *w) {
            conflicts.push((*w, ContentionKind::ReadWrite));
        }
    }

    conflicts
}

fn systems_can_run_parallel(a: &SystemDescriptor, b: &SystemDescriptor) -> bool {
    systems_conflict(a, b).is_empty()
}

/// Builds a topology report by analyzing system descriptors.
/// Groups independent systems into parallel_groups, serialized_systems for those with conflicts,
/// and collects contention points.
pub fn build_topology_report(systems: &[&dyn EngineSystem]) -> JobTopologyReport {
    let descriptors: Vec<SystemDescriptor> = systems.iter().map(|s| s.descriptor()).collect();
    let names: Vec<String> = descriptors.iter().map(|d| d.name.to_string()).collect();

    let n = descriptors.len();
    let mut conflict_matrix = vec![vec![false; n]; n];
    let mut contention_points = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            if systems_can_run_parallel(&descriptors[i], &descriptors[j]) {
                continue;
            }
            let conflicts = systems_conflict(&descriptors[i], &descriptors[j]);
            if !conflicts.is_empty() {
                conflict_matrix[i][j] = true;
                conflict_matrix[j][i] = true;
                for (tid, kind) in conflicts {
                    contention_points.push(ContentionPoint {
                        resource: format!("{:?}", tid),
                        systems: vec![names[i].clone(), names[j].clone()],
                        kind,
                    });
                }
            }
        }
    }

    let mut assigned = vec![false; n];
    let mut parallel_groups = Vec::new();

    for i in 0..n {
        if assigned[i] {
            continue;
        }
        let mut group = vec![names[i].clone()];
        assigned[i] = true;

        for j in (i + 1)..n {
            if assigned[j] {
                continue;
            }
            let can_join = group.iter().all(|gi| {
                let gi_idx = names.iter().position(|n| n == gi).unwrap();
                !conflict_matrix[gi_idx][j]
            });
            if can_join {
                group.push(names[j].clone());
                assigned[j] = true;
            }
        }
        parallel_groups.push(group);
    }

    let serialized_systems: Vec<String> = parallel_groups
        .iter()
        .filter(|g| g.len() == 1)
        .flat_map(|g| g.clone())
        .collect();

    let fence_count = parallel_groups.len().saturating_sub(1);

    JobTopologyReport {
        total_jobs: n,
        parallel_groups,
        serialized_systems,
        fence_count,
        contention_points,
    }
}
