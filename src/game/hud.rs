use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HudState {
    pub show_health: bool,
    pub show_stamina: bool,
    pub show_compass: bool,
    pub show_minimap: bool,
    pub show_crosshair: bool,
    pub show_interaction_prompt: bool,
    pub show_quest_tracker: bool,
    pub interaction_text: String,
    pub active_quest_name: String,
    pub active_quest_progress: String,
    pub notification_queue: Vec<HudNotification>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HudNotification {
    pub text: String,
    pub duration: f32,
    pub elapsed: f32,
    pub kind: NotificationKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NotificationKind {
    Info,
    Warning,
    QuestUpdate,
    ItemPickup,
    Damage,
}

impl HudState {
    pub fn new() -> Self {
        Self {
            show_health: true,
            show_stamina: true,
            show_compass: true,
            show_minimap: false,
            show_crosshair: true,
            show_interaction_prompt: false,
            show_quest_tracker: true,
            ..Default::default()
        }
    }

    pub fn push_notification(&mut self, text: &str, kind: NotificationKind) {
        self.notification_queue.push(HudNotification {
            text: text.to_string(),
            duration: 3.0,
            elapsed: 0.0,
            kind,
        });
    }

    pub fn tick(&mut self, dt: f32) {
        for notif in &mut self.notification_queue {
            notif.elapsed += dt;
        }
        self.notification_queue.retain(|n| n.elapsed < n.duration);
    }

    pub fn set_interaction(&mut self, text: &str) {
        self.show_interaction_prompt = true;
        self.interaction_text = text.to_string();
    }

    pub fn clear_interaction(&mut self) {
        self.show_interaction_prompt = false;
        self.interaction_text.clear();
    }

    pub fn set_quest(&mut self, name: &str, progress: &str) {
        self.active_quest_name = name.to_string();
        self.active_quest_progress = progress.to_string();
    }
}
