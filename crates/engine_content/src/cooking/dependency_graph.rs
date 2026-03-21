use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

pub type ArtifactId = u64;

#[derive(Clone, Debug)]
pub struct ContentProvenance {
    pub source_path: PathBuf,
    pub source_hash: u64,
    pub import_settings_hash: u64,
    pub cook_variant: String,
    pub build_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct ArtifactManifestEntry {
    pub id: ArtifactId,
    pub output_path: PathBuf,
    pub size_bytes: u64,
    pub provenance: ContentProvenance,
}

#[derive(Clone, Debug, Default)]
pub struct CookedArtifactManifest {
    pub entries: Vec<ArtifactManifestEntry>,
}

impl CookedArtifactManifest {
    pub fn add(&mut self, entry: ArtifactManifestEntry) {
        self.entries.push(entry);
    }

    pub fn total_size(&self) -> u64 {
        self.entries.iter().map(|e| e.size_bytes).sum()
    }

    pub fn find_by_source(&self, source_hash: u64) -> Vec<&ArtifactManifestEntry> {
        self.entries
            .iter()
            .filter(|e| e.provenance.source_hash == source_hash)
            .collect()
    }
}

pub struct ContentDependencyGraph {
    edges: HashMap<ArtifactId, Vec<ArtifactId>>,
    reverse: HashMap<ArtifactId, Vec<ArtifactId>>,
    provenance: HashMap<ArtifactId, ContentProvenance>,
    dirty: HashSet<ArtifactId>,
}

impl ContentDependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            reverse: HashMap::new(),
            provenance: HashMap::new(),
            dirty: HashSet::new(),
        }
    }

    pub fn add_dependency(&mut self, artifact: ArtifactId, depends_on: ArtifactId) {
        self.edges.entry(artifact).or_default().push(depends_on);
        self.reverse.entry(depends_on).or_default().push(artifact);
    }

    pub fn set_provenance(&mut self, id: ArtifactId, prov: ContentProvenance) {
        self.provenance.insert(id, prov);
    }

    pub fn get_provenance(&self, id: ArtifactId) -> Option<&ContentProvenance> {
        self.provenance.get(&id)
    }

    pub fn mark_dirty(&mut self, id: ArtifactId) {
        self.dirty.insert(id);
    }

    pub fn is_dirty(&self, id: ArtifactId) -> bool {
        self.dirty.contains(&id)
    }

    pub fn clear_dirty(&mut self, id: ArtifactId) {
        self.dirty.remove(&id);
    }

    pub fn invalidate(&self, changed: ArtifactId) -> Vec<ArtifactId> {
        let mut invalidated = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(changed);

        while let Some(id) = queue.pop_front() {
            if invalidated.insert(id) {
                if let Some(dependents) = self.reverse.get(&id) {
                    for &dep in dependents {
                        queue.push_back(dep);
                    }
                }
            }
        }

        invalidated.into_iter().collect()
    }

    pub fn invalidate_and_mark_dirty(&mut self, changed: ArtifactId) -> Vec<ArtifactId> {
        let affected = self.invalidate(changed);
        for &id in &affected {
            self.dirty.insert(id);
        }
        affected
    }

    pub fn dirty_build_order(&self) -> Vec<ArtifactId> {
        let full_order = self.build_order();
        full_order
            .into_iter()
            .filter(|id| self.dirty.contains(id))
            .collect()
    }

    pub fn build_order(&self) -> Vec<ArtifactId> {
        let mut in_degree: HashMap<ArtifactId, usize> = HashMap::new();
        for &id in self.edges.keys() {
            in_degree.entry(id).or_insert(0);
        }
        for deps in self.edges.values() {
            for &dep in deps {
                in_degree.entry(dep).or_insert(0);
            }
        }
        for (id, deps) in &self.edges {
            for _dep in deps {
                *in_degree.entry(*id).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<ArtifactId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(dependents) = self.reverse.get(&id) {
                for &dep in dependents {
                    let deg = in_degree.get_mut(&dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
        result
    }

    pub fn generate_manifest(&self) -> CookedArtifactManifest {
        let mut manifest = CookedArtifactManifest::default();
        for (&id, prov) in &self.provenance {
            manifest.add(ArtifactManifestEntry {
                id,
                output_path: prov.source_path.clone(),
                size_bytes: 0,
                provenance: prov.clone(),
            });
        }
        manifest
    }

    pub fn node_count(&self) -> usize {
        let mut all: HashSet<ArtifactId> = HashSet::new();
        for (&id, deps) in &self.edges {
            all.insert(id);
            for &d in deps {
                all.insert(d);
            }
        }
        all.len()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }
}

impl Default for ContentDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
