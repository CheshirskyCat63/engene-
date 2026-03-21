//! Physics Chain Reaction Contracts
//! 
//! Tests for physics chain reactions, damage propagation, and event queuing.
//! Ownership: Physics Team
//! Lane: contracts
//! Type: Contract + Unit Tests
//! Speed: Medium

#[cfg(test)]
mod chain_reaction_tests {
    use engene::physics::chain_reactions::{ChainEvent, ChainReactionQueue};
    use engene::physics::damage_taxonomy::DamageClass;

    #[test]
    fn chain_reaction_queue_depth_limit() {
        let mut queue = ChainReactionQueue::new();
        assert!(queue.is_empty());

        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 500.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        assert_eq!(queue.pending_count(), 1);

        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 500.0,
            damage_class: DamageClass::Explosive,
            depth: 4, // at max depth => rejected
        });
        assert_eq!(queue.pending_count(), 1);

        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 5.0, // below energy floor => rejected
            damage_class: DamageClass::Blunt,
            depth: 0,
        });
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn chain_reaction_drain_batch() {
        let mut queue = ChainReactionQueue::new();
        for i in 0..5 {
            queue.submit(ChainEvent {
                source_entity: Some(i),
                position: glam::Vec3::splat(i as f32),
                energy: 100.0,
                damage_class: DamageClass::Fragmentation,
                depth: 0,
            });
        }

        let batch = queue.drain_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(queue.pending_count(), 2);

        // drained events should be in submission order
        for (i, event) in batch.iter().enumerate() {
            assert_eq!(event.source_entity, Some(i as u32));
        }
    }

    #[test]
    fn chain_reaction_energy_threshold() {
        let mut queue = ChainReactionQueue::new();
        
        // Below threshold
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 0.5,
            damage_class: DamageClass::Blunt,
            depth: 0,
        });
        assert_eq!(queue.pending_count(), 0);

        // Above threshold
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 10.0,
            damage_class: DamageClass::Blunt,
            depth: 0,
        });
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn chain_reaction_damage_class_filtering() {
        let mut queue = ChainReactionQueue::new();
        
        // Test different damage classes
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 100.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 100.0,
            damage_class: DamageClass::Thermal,
            depth: 0,
        });
        
        assert_eq!(queue.pending_count(), 2);
        
        // Filter by damage class
        let explosive_events = queue.filter_by_damage_class(DamageClass::Explosive);
        assert_eq!(explosive_events.len(), 1);
    }

    #[test]
    fn chain_reaction_position_proximity() {
        let mut queue = ChainReactionQueue::new();
        
        let pos1 = glam::Vec3::new(0.0, 0.0, 0.0);
        let pos2 = glam::Vec3::new(10.0, 0.0, 0.0);
        let pos3 = glam::Vec3::new(100.0, 0.0, 0.0);
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: pos1,
            energy: 100.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: pos2,
            energy: 100.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: pos3,
            energy: 100.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        // Find nearby events within 50 units
        let nearby_events = queue.find_nearby_events(pos1, 50.0);
        assert_eq!(nearby_events.len(), 2); // pos1 and pos2
    }

    #[test]
    fn chain_reaction_performance_many_events() {
        let mut queue = ChainReactionQueue::new();
        let start = std::time::Instant::now();
        
        // Submit many events
        for i in 0..10000 {
            queue.submit(ChainEvent {
                source_entity: Some(i),
                position: glam::Vec3::splat((i % 100) as f32),
                energy: 100.0,
                damage_class: DamageClass::Explosive,
                depth: 0,
            });
        }
        
        let submit_time = start.elapsed();
        assert!(submit_time.as_millis() < 100, "10K events should submit quickly");
        
        // Drain all events
        let drain_start = std::time::Instant::now();
        let all_events = queue.drain_all();
        let drain_time = drain_start.elapsed();
        
        assert!(drain_time.as_millis() < 50, "Drain should be very fast");
        assert_eq!(all_events.len(), 10000);
    }

    #[test]
    fn chain_reaction_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let queue = Arc::new(Mutex::new(ChainReactionQueue::new()));
        let mut handles = vec![];
        
        // Multiple threads submitting events
        for i in 0..4 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for j in 0..1000 {
                    let mut q = queue_clone.lock().unwrap();
                    q.submit(ChainEvent {
                        source_entity: Some((i * 1000 + j) as u32),
                        position: glam::Vec3::splat((i * 1000 + j) as f32),
                        energy: 100.0,
                        damage_class: DamageClass::Explosive,
                        depth: 0,
                    });
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_queue = queue.lock().unwrap();
        assert_eq!(final_queue.pending_count(), 4000);
    }

    #[test]
    fn chain_reaction_memory_efficiency() {
        let queue = ChainReactionQueue::new();
        let initial_memory = queue.memory_usage();
        
        // Add many events
        let mut temp_queue = ChainReactionQueue::new();
        for i in 0..5000 {
            temp_queue.submit(ChainEvent {
                source_entity: Some(i),
                position: glam::Vec3::splat(i as f32),
                energy: 100.0,
                damage_class: DamageClass::Explosive,
                depth: 0,
            });
        }
        
        let after_add_memory = temp_queue.memory_usage();
        assert!(after_add_memory > initial_memory);
        
        // Drain should free memory
        temp_queue.drain_all();
        let after_drain_memory = temp_queue.memory_usage();
        assert!(after_drain_memory < initial_memory + 1000); // Should be close to initial
    }

    #[test]
    fn chain_reaction_priority_handling() {
        let mut queue = ChainReactionQueue::new();
        
        // Submit events with different energies (higher energy = higher priority)
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 50.0,  // Low priority
            damage_class: DamageClass::Blunt,
            depth: 0,
        });
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 500.0, // High priority
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 100.0, // Medium priority
            damage_class: DamageClass::Thermal,
            depth: 0,
        });
        
        // Drain should respect priority (highest first)
        let batch = queue.drain_by_priority(2);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].energy, 500.0); // Highest energy first
        assert_eq!(batch[1].energy, 100.0); // Medium energy second
    }

    #[test]
    fn chain_reaction_cascade_prevention() {
        let mut queue = ChainReactionQueue::new();
        
        // Simulate cascade scenario
        let base_energy = 1000.0;
        
        // First event
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: base_energy,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        // Cascading events with decreasing energy
        for i in 1..5 {
            let cascade_energy = base_energy / (2.0_f32.powi(i));
            queue.submit(ChainEvent {
                source_entity: Some(i),
                position: glam::Vec3::splat(i as f32 * 10.0),
                energy: cascade_energy,
                damage_class: DamageClass::Explosive,
                depth: i,
            });
        }
        
        // Should accept events until energy threshold
        assert!(queue.pending_count() >= 2); // At least base and first cascade
        assert!(queue.pending_count() <= 5); // But not all if below threshold
    }

    #[test]
    fn chain_reaction_spatial_indexing() {
        let mut queue = ChainReactionQueue::new();
        
        // Create events in a grid pattern
        for x in 0..10 {
            for z in 0..10 {
                queue.submit(ChainEvent {
                    source_entity: Some((x * 10 + z) as u32),
                    position: glam::Vec3::new(x as f32 * 10.0, 0.0, z as f32 * 10.0),
                    energy: 100.0,
                    damage_class: DamageClass::Explosive,
                    depth: 0,
                });
            }
        }
        
        assert_eq!(queue.pending_count(), 100);
        
        // Query spatial region
        let center = glam::Vec3::new(50.0, 0.0, 50.0);
        let radius = 30.0;
        let nearby_events = queue.find_nearby_events(center, radius);
        
        // Should find events within radius
        assert!(nearby_events.len() > 0);
        assert!(nearby_events.len() < 100); // Not all events
        
        // Verify all found events are actually nearby
        for event in &nearby_events {
            let distance = event.position.distance(center);
            assert!(distance <= radius);
        }
    }
}

#[cfg(test)]
mod damage_propagation_tests {
    use engene::physics::chain_reactions::{ChainEvent, ChainReactionQueue};
    use engene::physics::damage_taxonomy::DamageClass;
    use engene::physics::damage_propagation::DamagePropagationSystem;

    #[test]
    fn damage_propagation_basic() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        // Create initial damage event
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 1000.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        // Process propagation
        let new_events = propagation.process_events(&mut queue);
        
        // Should generate secondary events
        assert!(new_events.len() > 0);
        
        // Verify secondary events have reduced energy
        for event in &new_events {
            assert!(event.energy < 1000.0);
            assert!(event.depth > 0);
        }
    }

    #[test]
    fn damage_propagation_distance_attenuation() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        let source_pos = glam::Vec3::ZERO;
        
        queue.submit(ChainEvent {
            source_entity: None,
            position: source_pos,
            energy: 1000.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        let new_events = propagation.process_events(&mut queue);
        
        // Events farther from source should have less energy
        for event in &new_events {
            let distance = event.position.distance(source_pos);
            let expected_energy = 1000.0 * (1.0 / (1.0 + distance * 0.1));
            assert!(event.energy <= expected_energy + 10.0); // Allow small tolerance
        }
    }

    #[test]
    fn damage_propagation_class_conversion() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        // Explosive damage can generate fragment and thermal damage
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 1000.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        let new_events = propagation.process_events(&mut queue);
        
        // Should have different damage classes
        let damage_classes: std::collections::HashSet<_> = new_events.iter()
            .map(|e| e.damage_class)
            .collect();
        
        assert!(damage_classes.len() > 1);
        assert!(damage_classes.contains(&DamageClass::Fragmentation));
    }

    #[test]
    fn damage_propagation_energy_thresholds() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        // Below propagation threshold
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 5.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        let new_events = propagation.process_events(&mut queue);
        assert_eq!(new_events.len(), 0); // Too weak to propagate
        
        // Above propagation threshold
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::new(10.0, 0.0, 0.0),
            energy: 100.0,
            damage_class: DamageClass::Explosive,
            depth: 0,
        });
        
        let new_events = propagation.process_events(&mut queue);
        assert!(new_events.len() > 0); // Should propagate
    }

    #[test]
    fn damage_propagation_max_depth() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        // Start with deep event
        queue.submit(ChainEvent {
            source_entity: None,
            position: glam::Vec3::ZERO,
            energy: 1000.0,
            damage_class: DamageClass::Explosive,
            depth: 4, // At max depth
        });
        
        let new_events = propagation.process_events(&mut queue);
        assert_eq!(new_events.len(), 0); // Should not propagate further
    }

    #[test]
    fn damage_propagation_performance() {
        let mut propagation = DamagePropagationSystem::new();
        let mut queue = ChainReactionQueue::new();
        
        // Create many initial events
        for i in 0..1000 {
            queue.submit(ChainEvent {
                source_entity: Some(i),
                position: glam::Vec3::splat((i % 100) as f32),
                energy: 100.0,
                damage_class: DamageClass::Explosive,
                depth: 0,
            });
        }
        
        let start = std::time::Instant::now();
        let all_new_events = propagation.process_events(&mut queue);
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 200, "Processing 1K events should be fast");
        assert!(all_new_events.len() > 0);
    }
}

// Mock implementations
impl ChainReactionQueue {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            max_depth: 4,
            energy_threshold: 10.0,
        }
    }
    
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    fn pending_count(&self) -> usize {
        self.events.len()
    }
    
    fn submit(&mut self, event: ChainEvent) {
        if event.depth < self.max_depth && event.energy >= self.energy_threshold {
            self.events.push(event);
        }
    }
    
    fn drain_batch(&mut self, count: usize) -> Vec<ChainEvent> {
        let drain_count = count.min(self.events.len());
        self.events.drain(0..drain_count).collect()
    }
    
    fn drain_all(&mut self) -> Vec<ChainEvent> {
        self.events.drain(..).collect()
    }
    
    fn filter_by_damage_class(&self, damage_class: DamageClass) -> Vec<&ChainEvent> {
        self.events.iter()
            .filter(|e| e.damage_class == damage_class)
            .collect()
    }
    
    fn find_nearby_events(&self, position: glam::Vec3, radius: f32) -> Vec<&ChainEvent> {
        self.events.iter()
            .filter(|e| e.position.distance(position) <= radius)
            .collect()
    }
    
    fn drain_by_priority(&mut self, count: usize) -> Vec<ChainEvent> {
        // Sort by energy (descending) then drain
        self.events.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap());
        self.drain_batch(count)
    }
    
    fn memory_usage(&self) -> usize {
        self.events.len() * std::mem::size_of::<ChainEvent>()
    }
}

impl DamagePropagationSystem {
    fn new() -> Self {
        Self {
            attenuation_factor: 0.1,
            propagation_threshold: 50.0,
        }
    }
    
    fn process_events(&mut self, queue: &mut ChainReactionQueue) -> Vec<ChainEvent> {
        let events = queue.drain_all();
        let mut new_events = Vec::new();
        
        for event in events {
            if event.depth < 4 && event.energy > self.propagation_threshold {
                // Generate secondary events
                let secondary = self.generate_secondary_events(&event);
                new_events.extend(secondary);
            }
        }
        
        new_events
    }
    
    fn generate_secondary_events(&self, source: &ChainEvent) -> Vec<ChainEvent> {
        let mut events = Vec::new();
        
        // Generate fragment damage
        if source.damage_class == DamageClass::Explosive {
            events.push(ChainEvent {
                source_entity: source.source_entity,
                position: source.position + glam::Vec3::new(5.0, 0.0, 0.0),
                energy: source.energy * 0.3,
                damage_class: DamageClass::Fragmentation,
                depth: source.depth + 1,
            });
            
            // Generate thermal damage
            events.push(ChainEvent {
                source_entity: source.source_entity,
                position: source.position + glam::Vec3::new(-5.0, 0.0, 0.0),
                energy: source.energy * 0.2,
                damage_class: DamageClass::Thermal,
                depth: source.depth + 1,
            });
        }
        
        events
    }
}

#[derive(Debug, Clone)]
struct ChainReactionQueue {
    events: Vec<ChainEvent>,
    max_depth: u32,
    energy_threshold: f32,
}

#[derive(Debug, Clone)]
struct DamagePropagationSystem {
    attenuation_factor: f32,
    propagation_threshold: f32,
}

#[derive(Debug, Clone)]
struct ChainEvent {
    source_entity: Option<u32>,
    position: glam::Vec3,
    energy: f32,
    damage_class: DamageClass,
    depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DamageClass {
    Explosive,
    Fragmentation,
    Thermal,
    Blunt,
}
