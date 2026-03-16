use std::collections::HashMap;
use super::job::JobId;
use super::task_groups::TaskGroup;

pub struct FrameGraphNode {
    pub job_id: JobId,
    pub group: TaskGroup,
    pub dependencies: Vec<JobId>,
}

pub struct FrameGraph {
    nodes: Vec<FrameGraphNode>,
    next_id: JobId,
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_node(&mut self, group: TaskGroup, dependencies: Vec<JobId>) -> JobId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(FrameGraphNode {
            job_id: id,
            group,
            dependencies,
        });
        id
    }

    pub fn topological_order(&self) -> Vec<JobId> {
        let mut in_degree: HashMap<JobId, usize> = HashMap::new();
        for node in &self.nodes {
            in_degree.entry(node.job_id).or_insert(0);
            for dep in &node.dependencies {
                *in_degree.entry(*dep).or_insert(0) += 0;
            }
        }
        for node in &self.nodes {
            for _dep in &node.dependencies {
                *in_degree.entry(node.job_id).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<JobId> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        let mut result = Vec::new();

        while let Some(id) = queue.pop() {
            result.push(id);
            for node in &self.nodes {
                if node.dependencies.contains(&id) {
                    let deg = in_degree.get_mut(&node.job_id).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(node.job_id);
                    }
                }
            }
        }
        result
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.next_id = 0;
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
}

impl Default for FrameGraph {
    fn default() -> Self { Self::new() }
}
