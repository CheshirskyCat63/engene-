use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HierarchyNode {
    pub entity_id: u32,
    pub name: String,
    pub children: Vec<u32>,
    pub parent: Option<u32>,
    pub visible: bool,
    pub selected: bool,
}

pub struct SceneHierarchy {
    nodes: HashMap<u32, HierarchyNode>,
    root_entities: Vec<u32>,
    selected_entity: Option<u32>,
    filter: String,
}

impl SceneHierarchy {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_entities: Vec::new(),
            selected_entity: None,
            filter: String::new(),
        }
    }

    pub fn rebuild(&mut self, entities: &[(u32, String, Option<u32>)]) {
        self.nodes.clear();
        self.root_entities.clear();

        for (id, name, parent) in entities {
            self.nodes.insert(*id, HierarchyNode {
                entity_id: *id,
                name: name.clone(),
                children: Vec::new(),
                parent: *parent,
                visible: true,
                selected: self.selected_entity == Some(*id),
            });
        }

        let ids: Vec<u32> = self.nodes.keys().copied().collect();
        for id in &ids {
            let parent = self.nodes.get(id).and_then(|n| n.parent);
            if let Some(pid) = parent {
                if let Some(parent_node) = self.nodes.get_mut(&pid) {
                    parent_node.children.push(*id);
                }
            } else {
                self.root_entities.push(*id);
            }
        }
    }

    pub fn select(&mut self, entity_id: u32) {
        if let Some(old) = self.selected_entity {
            if let Some(node) = self.nodes.get_mut(&old) {
                node.selected = false;
            }
        }
        self.selected_entity = Some(entity_id);
        if let Some(node) = self.nodes.get_mut(&entity_id) {
            node.selected = true;
        }
    }

    pub fn selected(&self) -> Option<u32> {
        self.selected_entity
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_lowercase();
        for node in self.nodes.values_mut() {
            node.visible = self.filter.is_empty() || node.name.to_lowercase().contains(&self.filter);
        }
    }

    pub fn root_entities(&self) -> &[u32] {
        &self.root_entities
    }

    pub fn get_node(&self, id: u32) -> Option<&HierarchyNode> {
        self.nodes.get(&id)
    }

    pub fn entity_count(&self) -> usize {
        self.nodes.len()
    }

    #[cfg(feature = "debug_ui")]
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            let mut filter = self.filter.clone();
            if ui.text_edit_singleline(&mut filter).changed() {
                self.set_filter(&filter);
            }
        });
        ui.separator();

        let roots = self.root_entities.clone();
        for &id in &roots {
            self.draw_node(ui, id, 0);
        }
    }

    #[cfg(feature = "debug_ui")]
    fn draw_node(&mut self, ui: &mut egui::Ui, id: u32, depth: usize) {
        let (visible, label, selected, children) = {
            let Some(node) = self.nodes.get(&id) else { return };
            (
                node.visible,
                format!("{}{} [{}]", "  ".repeat(depth), node.name, id),
                node.selected,
                node.children.clone(),
            )
        };
        if !visible { return; }

        if ui.selectable_label(selected, &label).clicked() {
            self.select(id);
        }

        for child_id in children {
            self.draw_node(ui, child_id, depth + 1);
        }
    }
}

impl Default for SceneHierarchy {
    fn default() -> Self { Self::new() }
}
