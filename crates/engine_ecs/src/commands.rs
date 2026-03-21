use crate::persistent_id::Entity;
use std::any::Any;

pub enum ComponentOp {
    Add {
        entity: Entity,
        type_name: &'static str,
        data: Box<dyn Any + Send + Sync>,
    },
    Remove {
        entity: Entity,
        type_name: &'static str,
    },
}

pub struct SpawnCommand {
    pub ops: Vec<Box<dyn Any + Send + Sync>>,
}

pub struct CommandBuffer {
    spawns: Vec<SpawnCommand>,
    despawns: Vec<Entity>,
    component_ops: Vec<ComponentOp>,
    deferred_events: Vec<Box<dyn Any + Send + Sync>>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            spawns: Vec::new(),
            despawns: Vec::new(),
            component_ops: Vec::new(),
            deferred_events: Vec::new(),
        }
    }

    pub fn spawn(&mut self) -> &mut SpawnCommand {
        self.spawns.push(SpawnCommand { ops: Vec::new() });
        self.spawns.last_mut().unwrap()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.despawns.push(entity);
    }

    pub fn add_component<T: Any + Send + Sync>(&mut self, entity: Entity, data: T) {
        self.component_ops.push(ComponentOp::Add {
            entity,
            type_name: std::any::type_name::<T>(),
            data: Box::new(data),
        });
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        self.component_ops.push(ComponentOp::Remove {
            entity,
            type_name: std::any::type_name::<T>(),
        });
    }

    pub fn emit_event<E: Any + Send + Sync>(&mut self, event: E) {
        self.deferred_events.push(Box::new(event));
    }

    pub fn spawn_count(&self) -> usize {
        self.spawns.len()
    }
    pub fn despawn_count(&self) -> usize {
        self.despawns.len()
    }
    pub fn component_op_count(&self) -> usize {
        self.component_ops.len()
    }
    pub fn event_count(&self) -> usize {
        self.deferred_events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.spawns.is_empty()
            && self.despawns.is_empty()
            && self.component_ops.is_empty()
            && self.deferred_events.is_empty()
    }

    pub fn take_spawns(&mut self) -> Vec<SpawnCommand> {
        std::mem::take(&mut self.spawns)
    }
    pub fn take_despawns(&mut self) -> Vec<Entity> {
        std::mem::take(&mut self.despawns)
    }
    pub fn take_component_ops(&mut self) -> Vec<ComponentOp> {
        std::mem::take(&mut self.component_ops)
    }
    pub fn take_events(&mut self) -> Vec<Box<dyn Any + Send + Sync>> {
        std::mem::take(&mut self.deferred_events)
    }

    pub fn clear(&mut self) {
        self.spawns.clear();
        self.despawns.clear();
        self.component_ops.clear();
        self.deferred_events.clear();
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_command() {
        let mut buf = CommandBuffer::new();

        buf.spawn();
        buf.spawn();

        assert_eq!(buf.spawn_count(), 2);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_despawn_command() {
        let mut buf = CommandBuffer::new();

        buf.despawn(1);
        buf.despawn(2);
        buf.despawn(3);

        assert_eq!(buf.despawn_count(), 3);
    }

    #[test]
    fn test_add_and_remove_components() {
        let mut buf = CommandBuffer::new();

        buf.add_component(1u64, 42i32);
        buf.add_component(1u64, "test".to_string());
        buf.remove_component::<f32>(2u64);

        assert_eq!(buf.component_op_count(), 3);
    }

    #[test]
    fn test_deferred_events() {
        let mut buf = CommandBuffer::new();

        buf.emit_event(42u32);
        buf.emit_event("hello".to_string());

        assert_eq!(buf.event_count(), 2);
    }

    #[test]
    fn test_take_methods() {
        let mut buf = CommandBuffer::new();

        buf.spawn();
        buf.despawn(1);
        buf.add_component(2u64, 100i32);
        buf.emit_event(999u64);

        let spawns = buf.take_spawns();
        let despawns = buf.take_despawns();
        let ops = buf.take_component_ops();
        let events = buf.take_events();

        assert_eq!(spawns.len(), 1);
        assert_eq!(despawns.len(), 1);
        assert_eq!(ops.len(), 1);
        assert_eq!(events.len(), 1);

        // After take, buffer should be empty
        assert!(buf.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut buf = CommandBuffer::new();

        buf.spawn();
        buf.despawn(1);
        buf.add_component(2u64, 100i32);
        buf.emit_event(999u64);

        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.spawn_count(), 0);
        assert_eq!(buf.despawn_count(), 0);
        assert_eq!(buf.component_op_count(), 0);
        assert_eq!(buf.event_count(), 0);
    }
}
