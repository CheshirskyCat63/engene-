//! Quest Board — SDK panel showing active, completed, and failed quests.

#[derive(Debug, Clone)]
pub struct QuestBoardEntry {
    pub quest_id: u32,
    pub quest_type: String,
    pub status: String,
    pub giver_name: String,
    pub assignee_name: Option<String>,
    pub progress: String,
    pub reward: f32,
}

pub struct QuestBoardPanel {
    pub entries: Vec<QuestBoardEntry>,
    pub total_generated: u32,
    pub total_completed: u32,
    pub total_failed: u32,
    pub total_expired: u32,
}

impl QuestBoardPanel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_generated: 0,
            total_completed: 0,
            total_failed: 0,
            total_expired: 0,
        }
    }

    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|e| e.status == "Active").count()
    }

    pub fn completion_rate(&self) -> f32 {
        let total = self.total_completed + self.total_failed + self.total_expired;
        if total == 0 {
            return 0.0;
        }
        self.total_completed as f32 / total as f32
    }
}

impl QuestBoardPanel {
    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("Quest Board").default_width(450.0).show(ctx, |ui| {
            ui.label(format!(
                "Generated: {}  Completed: {}  Failed: {}  Expired: {}  Rate: {:.0}%",
                self.total_generated, self.total_completed, self.total_failed,
                self.total_expired, self.completion_rate() * 100.0
            ));
            ui.label(format!("Active: {}", self.active_count()));
            ui.separator();
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                for entry in &self.entries {
                    ui.horizontal(|ui| {
                        ui.monospace(format!("#{}", entry.quest_id));
                        ui.label(&entry.quest_type);
                        ui.label(format!("[{}]", entry.status));
                        ui.label(format!("reward:{:.0}", entry.reward));
                    });
                }
            });
        });
    }
}

impl Default for QuestBoardPanel {
    fn default() -> Self {
        Self::new()
    }
}
