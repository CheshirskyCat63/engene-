pub struct OverlayState {
    pub nav_mesh: bool,
    pub cover_map: bool,
    pub destruction_graph: bool,
    pub ai_debug_lines: bool,
    pub terrain_damage: bool,
    pub simulation_levels: bool,
    pub territory_map: bool,
    pub fire_grid: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            nav_mesh: false,
            cover_map: false,
            destruction_graph: false,
            ai_debug_lines: false,
            terrain_damage: false,
            simulation_levels: false,
            territory_map: false,
            fire_grid: false,
        }
    }
}

pub fn draw_overlay_panel(ctx: &egui::Context, state: &mut OverlayState) {
    egui::Window::new("World Overlays")
        .collapsible(true)
        .show(ctx, |ui| {
            ui.checkbox(&mut state.nav_mesh, "Nav Mesh");
            ui.checkbox(&mut state.cover_map, "Cover Map");
            ui.checkbox(&mut state.destruction_graph, "Destruction Graph");
            ui.checkbox(&mut state.ai_debug_lines, "AI Debug Lines");
            ui.checkbox(&mut state.terrain_damage, "Terrain Damage");
            ui.checkbox(&mut state.simulation_levels, "Simulation Levels");
            ui.checkbox(&mut state.territory_map, "Territory Map");
            ui.checkbox(&mut state.fire_grid, "Fire Grid");
        });
}
