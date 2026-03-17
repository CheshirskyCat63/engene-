use crate::core::ecs::Ecs;

#[derive(Clone, Debug)]
pub enum InspectorEdit {
    SetTransform { entity: u64, x: f32, y: f32 },
    SetHealth { entity: u64, value: f32 },
    SetHunger { entity: u64, value: f32 },
    SetThirst { entity: u64, value: f32 },
    SetEnergy { entity: u64, value: f32 },
}

pub struct InspectorState {
    pub selected_entity: Option<u64>,
    pub search_filter: String,
    pub pending_edits: Vec<InspectorEdit>,
    pub editing_enabled: bool,
    edit_x: f32,
    edit_y: f32,
    edit_health: f32,
    edit_hunger: f32,
    edit_thirst: f32,
    edit_energy: f32,
    synced_entity: Option<u64>,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            selected_entity: None,
            search_filter: String::new(),
            pending_edits: Vec::new(),
            editing_enabled: true,
            edit_x: 0.0,
            edit_y: 0.0,
            edit_health: 1.0,
            edit_hunger: 0.0,
            edit_thirst: 0.0,
            edit_energy: 1.0,
            synced_entity: None,
        }
    }
}

pub fn draw_inspector(ctx: &egui::Context, ecs: &Ecs, state: &mut InspectorState) {
    egui::SidePanel::left("inspector").default_width(300.0).show(ctx, |ui| {
        ui.heading("Entity Inspector");
        ui.checkbox(&mut state.editing_enabled, "Enable Editing");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_filter);
        });

        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            for &entity in &ecs.alive {
                let name = ecs.get_name(entity)
                    .map(|n| n.0.as_str())
                    .unwrap_or("unnamed");

                if !state.search_filter.is_empty() && !name.to_lowercase().contains(&state.search_filter.to_lowercase()) {
                    continue;
                }

                let selected = state.selected_entity == Some(entity);
                if ui.selectable_label(selected, format!("[{}] {}", entity, name)).clicked() {
                    state.selected_entity = Some(entity);
                }
            }
        });

        ui.separator();

        if let Some(entity) = state.selected_entity {
            if state.synced_entity != Some(entity) {
                if let Some(t) = ecs.get_transform(entity) {
                    state.edit_x = t.x;
                    state.edit_y = t.y;
                }
                if let Some(n) = ecs.get_needs(entity) {
                    state.edit_health = n.health;
                    state.edit_hunger = n.hunger;
                    state.edit_thirst = n.thirst;
                    state.edit_energy = n.energy;
                }
                state.synced_entity = Some(entity);
            }

            ui.heading(format!("Entity {}", entity));

            if let Some(pid) = ecs.identity.persistent_id_of(entity) {
                ui.label(format!("PID: {:?}", pid));
            }
            if let Some(name) = ecs.get_name(entity) {
                ui.label(format!("Name: {}", name.0));
            }
            if let Some(kind) = ecs.get_kind(entity) {
                ui.label(format!("Kind: {:?}", kind));
            }

            if ecs.get_transform(entity).is_some() {
                ui.collapsing("Transform", |ui| {
                    if state.editing_enabled {
                        let mut changed = false;
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_x).speed(1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_y).speed(1.0)).changed();
                        });
                        if changed {
                            state.pending_edits.push(InspectorEdit::SetTransform {
                                entity,
                                x: state.edit_x,
                                y: state.edit_y,
                            });
                        }
                    } else if let Some(t) = ecs.get_transform(entity) {
                        ui.label(format!("Position: ({:.1}, {:.1})", t.x, t.y));
                        ui.label(format!("Cell: ({}, {})", t.cell_x, t.cell_y));
                    }
                });
            }
            if let Some(sim) = ecs.sim_levels.get(&entity) {
                ui.label(format!("Sim Level: {:?}", sim.level));
            }
            if ecs.get_needs(entity).is_some() {
                ui.collapsing("Personal Needs", |ui| {
                    if state.editing_enabled {
                        let mut changed = false;
                        ui.horizontal(|ui| {
                            ui.label("Health:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_health).speed(0.01).range(0.0..=1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Hunger:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_hunger).speed(0.01).range(0.0..=1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Thirst:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_thirst).speed(0.01).range(0.0..=1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Energy:");
                            changed |= ui.add(egui::DragValue::new(&mut state.edit_energy).speed(0.01).range(0.0..=1.0)).changed();
                        });
                        if changed {
                            state.pending_edits.push(InspectorEdit::SetHealth {
                                entity,
                                value: state.edit_health,
                            });
                            state.pending_edits.push(InspectorEdit::SetHunger {
                                entity,
                                value: state.edit_hunger,
                            });
                            state.pending_edits.push(InspectorEdit::SetThirst {
                                entity,
                                value: state.edit_thirst,
                            });
                            state.pending_edits.push(InspectorEdit::SetEnergy {
                                entity,
                                value: state.edit_energy,
                            });
                        }
                    } else if let Some(needs) = ecs.get_needs(entity) {
                        ui.label(format!("Health: {:.2}", needs.health));
                        ui.label(format!("Hunger: {:.2}", needs.hunger));
                        ui.label(format!("Thirst: {:.2}", needs.thirst));
                        ui.label(format!("Energy: {:.2}", needs.energy));
                        ui.label(format!("Fear:   {:.2}", needs.fear));
                    }
                });
            }
            if let Some(ai) = ecs.ai_states.get(&entity) {
                ui.label(format!("AI State: {:?}", ai));
            }
            if let Some(emotions) = ecs.get_emotions(entity) {
                ui.collapsing("Emotions", |ui| {
                    ui.label(format!("Fear:     {:.2}", emotions.fear));
                    ui.label(format!("Anger:    {:.2}", emotions.anger));
                    ui.label(format!("Joy:      {:.2}", emotions.joy));
                    ui.label(format!("Grief:    {:.2}", emotions.grief));
                    ui.label(format!("Disgust:  {:.2}", emotions.disgust));
                    ui.label(format!("Surprise: {:.2}", emotions.surprise));
                    ui.label(format!("Longing:  {:.2}", emotions.longing));
                });
            }
            if let Some(inv) = ecs.inventories.get(&entity) {
                ui.collapsing("Inventory", |ui| {
                    if inv.items.is_empty() {
                        ui.label("(empty)");
                    } else {
                        for item in &inv.items {
                            ui.label(format!("{}: ${:.0}", item.name, item.value));
                        }
                    }
                });
            }
            if let Some(econ) = ecs.get_npc_economy(entity) {
                ui.collapsing("Economy", |ui| {
                    ui.label(format!("Money: ${:.0}", econ.money));
                    ui.label(format!("Monthly required: ${:.0}", econ.monthly_required));
                    ui.label(format!("Desperation: {:.2}", econ.desperation));
                });
            }
        }
    });
}
