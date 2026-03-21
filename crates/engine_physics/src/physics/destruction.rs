use crate::core::ecs::Entity;
use crate::physics::ballistics::MaterialId;
use crate::physics::building::StructuralSection;
use glam::Vec3;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DestructionNode {
    pub id: u32,
    pub position: Vec3,
    pub mass: f32,
    pub material: MaterialId,
    pub accumulated_stress: f32,
}

#[derive(Clone, Debug)]
pub struct DestructionLink {
    pub a: u32,
    pub b: u32,
    pub strength: f32,
    pub fatigue: f32,
    pub broken: bool,
}

#[derive(Clone, Debug)]
pub struct DestructibleObject {
    pub nodes: Vec<DestructionNode>,
    pub links: Vec<DestructionLink>,
    pub entity: Entity,
    pub total_strength: f32,
}

impl DestructibleObject {
    pub fn new(entity: Entity, nodes: Vec<DestructionNode>, links: Vec<DestructionLink>) -> Self {
        let total_strength: f32 = links.iter().map(|l| l.strength).sum();
        Self {
            nodes,
            links,
            entity,
            total_strength,
        }
    }

    pub fn apply_impulse(&mut self, position: Vec3, energy: f32) -> Vec<DestructionEvent> {
        let mut events = Vec::new();

        let nearest = self.nodes.iter_mut().min_by(|a, b| {
            let da = (a.position - position).length_squared();
            let db = (b.position - position).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

        let nearest_id = match nearest {
            Some(n) => {
                n.accumulated_stress += energy;
                n.id
            }
            None => return events,
        };

        self.propagate_stress(nearest_id, energy);

        for link in &mut self.links {
            if link.broken {
                continue;
            }
            let threshold = link.strength * (1.0 - link.fatigue);
            let node_a_stress = self
                .nodes
                .iter()
                .find(|n| n.id == link.a)
                .map_or(0.0, |n| n.accumulated_stress);
            let node_b_stress = self
                .nodes
                .iter()
                .find(|n| n.id == link.b)
                .map_or(0.0, |n| n.accumulated_stress);
            let max_stress = node_a_stress.max(node_b_stress);

            if max_stress > threshold {
                link.broken = true;
                link.fatigue = 1.0;
                events.push(DestructionEvent::LinkBroken {
                    entity: self.entity,
                    link_a: link.a,
                    link_b: link.b,
                });
            } else {
                link.fatigue = (link.fatigue + energy * 0.0001 / link.strength).min(0.99);
            }
        }

        let clusters = self.find_clusters();
        if clusters.len() > 1 {
            events.push(DestructionEvent::ObjectFragmented {
                entity: self.entity,
                cluster_count: clusters.len() as u32,
            });
        }

        events
    }

    fn propagate_stress(&mut self, source_id: u32, energy: f32) {
        let mut visited = vec![false; self.nodes.len()];
        let mut queue = std::collections::VecDeque::new();

        let source_idx = self.nodes.iter().position(|n| n.id == source_id);
        if let Some(idx) = source_idx {
            visited[idx] = true;
            queue.push_back((idx, energy));
        }

        while let Some((node_idx, remaining_energy)) = queue.pop_front() {
            if remaining_energy < 1.0 {
                continue;
            }
            let node_id = self.nodes[node_idx].id;

            for link in &self.links {
                if link.broken {
                    continue;
                }
                let neighbor_id = if link.a == node_id {
                    link.b
                } else if link.b == node_id {
                    link.a
                } else {
                    continue;
                };

                if let Some(ni) = self.nodes.iter().position(|n| n.id == neighbor_id) {
                    if !visited[ni] {
                        visited[ni] = true;
                        let transmitted = remaining_energy * 0.5 * (1.0 - link.fatigue);
                        self.nodes[ni].accumulated_stress += transmitted;
                        queue.push_back((ni, transmitted));
                    }
                }
            }
        }
    }

    pub fn find_clusters_public(&self) -> Vec<Vec<u32>> {
        self.find_clusters()
    }

    fn find_clusters(&self) -> Vec<Vec<u32>> {
        let mut visited = vec![false; self.nodes.len()];
        let mut clusters = Vec::new();

        for start_idx in 0..self.nodes.len() {
            if visited[start_idx] {
                continue;
            }

            let mut cluster = Vec::new();
            let mut stack = vec![start_idx];

            while let Some(idx) = stack.pop() {
                if visited[idx] {
                    continue;
                }
                visited[idx] = true;
                cluster.push(self.nodes[idx].id);

                let node_id = self.nodes[idx].id;
                for link in &self.links {
                    if link.broken {
                        continue;
                    }
                    let neighbor_id = if link.a == node_id {
                        link.b
                    } else if link.b == node_id {
                        link.a
                    } else {
                        continue;
                    };
                    if let Some(ni) = self.nodes.iter().position(|n| n.id == neighbor_id) {
                        if !visited[ni] {
                            stack.push(ni);
                        }
                    }
                }
            }

            clusters.push(cluster);
        }

        clusters
    }

    pub fn apply_statistical_damage(&mut self, energy: f32) -> f32 {
        let damage_ratio = (energy / self.total_strength).min(1.0);
        let links_to_break = (self.links.len() as f32 * damage_ratio) as usize;
        let mut broken = 0;
        for link in &mut self.links {
            if broken >= links_to_break {
                break;
            }
            if !link.broken {
                link.broken = true;
                broken += 1;
            }
        }
        damage_ratio
    }

    pub fn integrity(&self) -> f32 {
        let intact: f32 = self
            .links
            .iter()
            .filter(|l| !l.broken)
            .map(|l| l.strength)
            .sum();
        if self.total_strength > 0.0 {
            intact / self.total_strength
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug)]
pub enum DestructionEvent {
    LinkBroken {
        entity: Entity,
        link_a: u32,
        link_b: u32,
    },
    ObjectFragmented {
        entity: Entity,
        cluster_count: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DestructionLod {
    Full,
    Statistical,
    Frozen,
}

pub struct DestructionSystem {
    pub objects: Vec<DestructibleObject>,
    pub events: Vec<DestructionEvent>,
    entity_sections: HashMap<Entity, Vec<StructuralSection>>,
}

impl DestructionSystem {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            events: Vec::new(),
            entity_sections: HashMap::new(),
        }
    }

    pub fn register_object(&mut self, obj: DestructibleObject) {
        self.objects.push(obj);
    }

    pub fn register_sections(&mut self, entity: Entity, sections: Vec<StructuralSection>) {
        self.entity_sections.insert(entity, sections);
    }

    pub fn find_section_at(&self, entity: Entity, position: Vec3) -> Option<u32> {
        let sections = self.entity_sections.get(&entity)?;
        let obj = self.objects.iter().find(|o| o.entity == entity)?;

        let mut best_section = None;
        let mut best_dist = f32::MAX;

        for section in sections {
            for &nid in &section.node_ids {
                if let Some(node) = obj.nodes.iter().find(|n| n.id == nid) {
                    let d = (node.position - position).length_squared();
                    if d < best_dist {
                        best_dist = d;
                        best_section = Some(section.id);
                    }
                }
            }
        }
        best_section
    }

    pub fn apply_section_damage(&mut self, entity: Entity, section_id: u32, energy: f32) -> bool {
        let sections = match self.entity_sections.get_mut(&entity) {
            Some(s) => s,
            None => return false,
        };

        let section = match sections.iter_mut().find(|s| s.id == section_id) {
            Some(s) => s,
            None => return false,
        };

        section.integrity -= energy * 0.01;
        section.integrity = section.integrity.max(0.0);
        section.integrity <= 0.0
    }

    pub fn get_section_integrity(&self, entity: Entity, section_id: u32) -> f32 {
        self.entity_sections
            .get(&entity)
            .and_then(|secs| secs.iter().find(|s| s.id == section_id))
            .map(|s| s.integrity)
            .unwrap_or(0.0)
    }

    pub fn apply_impulse_at(&mut self, position: Vec3, energy: f32, lod: DestructionLod) {
        self.events.clear();

        let radius = 10.0;
        for obj in &mut self.objects {
            let close_enough = obj
                .nodes
                .iter()
                .any(|n| (n.position - position).length() < radius);
            if !close_enough {
                continue;
            }

            match lod {
                DestructionLod::Full => {
                    let evts = obj.apply_impulse(position, energy);
                    self.events.extend(evts);
                }
                DestructionLod::Statistical => {
                    obj.apply_statistical_damage(energy);
                }
                DestructionLod::Frozen => {}
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<DestructionEvent> {
        std::mem::take(&mut self.events)
    }
}
