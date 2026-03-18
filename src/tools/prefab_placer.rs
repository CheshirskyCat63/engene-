//! Prefab Placer — SDK tool for placing destructible content into the world.
//! Supports 9 prefab types, group placement, reset to defaults,
//! and "spawn from material preset" workflow.

use crate::content::prefabs::prefab_registry::PrefabRegistry;

const SANDBOX_PREFAB_TYPES: &[&str] = &[
    "tiled_wall",
    "destructible_table",
    "destructible_chair",
    "destructible_crate",
    "metal_cabinet",
    "barrel",
    "sandbag",
    "basement_light",
    "material_target",
];

const MATERIAL_PRESETS: &[&str] = &[
    "Wood", "Cloth", "Thatch", "Stone", "Metal", "Flesh", "Earth", "Tile", "Concrete", "Brick",
    "Glass", "Steel", "Sand", "Gravel", "Rubber", "Plastic",
];

pub struct PrefabPlacerState {
    pub selected_prefab: usize,
    pub selected_material: usize,
    pub placement_position: [f32; 3],
    pub placement_rotation: f32,
    pub placement_scale: f32,
    pub group_count: u32,
    pub group_spacing: f32,
    pub pending_placements: Vec<PrefabPlacement>,
    pub label_override: String,
}

#[derive(Clone, Debug)]
pub struct PrefabPlacement {
    pub prefab_name: String,
    pub position: [f32; 3],
    pub rotation: f32,
    pub scale: f32,
    pub material: Option<String>,
    pub label: Option<String>,
}

impl Default for PrefabPlacerState {
    fn default() -> Self {
        Self {
            selected_prefab: 0,
            selected_material: 0,
            placement_position: [25.0, 0.0, 25.0],
            placement_rotation: 0.0,
            placement_scale: 1.0,
            group_count: 1,
            group_spacing: 2.0,
            pending_placements: Vec::new(),
            label_override: String::new(),
        }
    }
}

impl PrefabPlacerState {
    pub fn reset_to_defaults(&mut self) {
        self.placement_position = [25.0, 0.0, 25.0];
        self.placement_rotation = 0.0;
        self.placement_scale = 1.0;
        self.group_count = 1;
        self.group_spacing = 2.0;
        self.label_override.clear();
    }

    fn queue_placement(&mut self) {
        let prefab_name = SANDBOX_PREFAB_TYPES[self.selected_prefab].to_string();
        let material = Some(MATERIAL_PRESETS[self.selected_material].to_string());
        let label = if self.label_override.is_empty() {
            None
        } else {
            Some(self.label_override.clone())
        };

        for i in 0..self.group_count {
            let offset = i as f32 * self.group_spacing;
            let pos = [
                self.placement_position[0] + offset,
                self.placement_position[1],
                self.placement_position[2],
            ];
            self.pending_placements.push(PrefabPlacement {
                prefab_name: prefab_name.clone(),
                position: pos,
                rotation: self.placement_rotation,
                scale: self.placement_scale,
                material: material.clone(),
                label: label.clone(),
            });
        }
    }
}

pub fn draw_prefab_placer(
    ctx: &egui::Context,
    state: &mut PrefabPlacerState,
    _registry: Option<&PrefabRegistry>,
) {
    egui::Window::new("Prefab Placer")
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.heading("Prefab Placement");
            ui.separator();

            egui::ComboBox::from_label("Prefab")
                .selected_text(SANDBOX_PREFAB_TYPES[state.selected_prefab])
                .show_ui(ui, |ui| {
                    for (i, name) in SANDBOX_PREFAB_TYPES.iter().enumerate() {
                        ui.selectable_value(&mut state.selected_prefab, i, *name);
                    }
                });

            ui.separator();
            ui.label("Material Preset");
            egui::ComboBox::from_label("Material")
                .selected_text(MATERIAL_PRESETS[state.selected_material])
                .show_ui(ui, |ui| {
                    for (i, name) in MATERIAL_PRESETS.iter().enumerate() {
                        ui.selectable_value(&mut state.selected_material, i, *name);
                    }
                });

            ui.separator();
            ui.label("Transform");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut state.placement_position[0]).speed(0.5));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut state.placement_position[1]).speed(0.5));
                ui.label("Z:");
                ui.add(egui::DragValue::new(&mut state.placement_position[2]).speed(0.5));
            });
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.add(
                    egui::DragValue::new(&mut state.placement_rotation)
                        .speed(1.0)
                        .suffix("°"),
                );
                ui.label("Scale:");
                ui.add(
                    egui::DragValue::new(&mut state.placement_scale)
                        .speed(0.05)
                        .range(0.1..=10.0),
                );
            });

            ui.separator();
            ui.label("Group Placement");
            ui.horizontal(|ui| {
                ui.label("Count:");
                ui.add(egui::DragValue::new(&mut state.group_count).range(1..=20));
                ui.label("Spacing:");
                ui.add(
                    egui::DragValue::new(&mut state.group_spacing)
                        .speed(0.1)
                        .suffix("m"),
                );
            });

            ui.separator();
            ui.label("Label Override (optional)");
            ui.text_edit_singleline(&mut state.label_override);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Place").clicked() {
                    state.queue_placement();
                }
                if ui.button("Reset to Defaults").clicked() {
                    state.reset_to_defaults();
                }
            });

            if !state.pending_placements.is_empty() {
                ui.separator();
                ui.label(format!(
                    "{} placement(s) queued",
                    state.pending_placements.len()
                ));
                if ui.button("Clear Queue").clicked() {
                    state.pending_placements.clear();
                }
            }
        });
}
