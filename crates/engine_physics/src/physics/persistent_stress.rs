use crate::core::ecs::Entity;
use crate::physics::damage_taxonomy::DamageClass;
use crate::physics::impact_event::StressEvent;

const MAX_ACTIVE_CONTACTS: usize = 128;

#[derive(Clone, Debug)]
pub struct PersistentContact {
    pub source_entity: Option<Entity>,
    pub target_entity: Entity,
    pub damage_class: DamageClass,
    pub intensity: f32,
    pub duration_active: f32,
    pub suspended: bool,
}

pub struct PersistentStressSystem {
    contacts: Vec<PersistentContact>,
}

impl PersistentStressSystem {
    pub fn new() -> Self {
        Self {
            contacts: Vec::with_capacity(MAX_ACTIVE_CONTACTS),
        }
    }

    pub fn add_contact(&mut self, contact: PersistentContact) {
        if self.contacts.len() < MAX_ACTIVE_CONTACTS {
            self.contacts.push(contact);
        }
    }

    pub fn remove_contact(&mut self, target: Entity, class: DamageClass) {
        self.contacts
            .retain(|c| !(c.target_entity == target && c.damage_class == class));
    }

    pub fn suspend_distant(&mut self, camera_pos_sq: f32, threshold_sq: f32) {
        for contact in &mut self.contacts {
            contact.suspended = camera_pos_sq > threshold_sq;
        }
    }

    pub fn tick(&mut self, dt: f32) -> Vec<StressEvent> {
        let mut events = Vec::new();

        for contact in &mut self.contacts {
            if contact.suspended {
                continue;
            }
            contact.duration_active += dt;

            events.push(StressEvent {
                target_entity: contact.target_entity,
                damage_class: contact.damage_class,
                intensity: contact.intensity,
                duration: dt,
                position: None,
                source_direction: None,
            });
        }

        self.contacts.retain(|c| c.intensity > 0.001);
        events
    }

    pub fn active_count(&self) -> usize {
        self.contacts.iter().filter(|c| !c.suspended).count()
    }
}
