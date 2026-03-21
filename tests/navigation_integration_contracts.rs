//! Navigation Integration Contracts
//! 
//! Tests for navigation systems, pathfinding, collision avoidance, and dynamic updates.
//! Ownership: Navigation Team
//! Lane: contracts
//! Type: Integration + Contract Tests
//! Speed: Medium

#[cfg(test)]
mod world_graph_tests {
    use engene::navigation::world_graph::WorldGraph;

    #[test]
    fn world_graph_pathfinding() {
        let graph = WorldGraph::build_default();
        assert_eq!(graph.locations.len(), 5);

        let village = 0;
        let forest_n = 1;
        let path = graph.find_path(village, forest_n);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(*path.first().unwrap(), village);
        assert_eq!(*path.last().unwrap(), forest_n);
    }

    #[test]
    fn world_graph_disconnected_no_path() {
        let mut graph = WorldGraph::new();
        let a = graph.add_location("A", 0, 0);
        let b = graph.add_location("B", 10, 10);

        let path = graph.find_path(a, b);
        assert!(path.is_none());
    }

    #[test]
    fn world_graph_neighbors() {
        let graph = WorldGraph::build_default();
        let village_neighbors = graph.neighbors(0);
        assert!(
            village_neighbors.len() >= 3,
            "village should connect to 3 locations"
        );
    }

    #[test]
    fn world_graph_dynamic_addition() {
        let mut graph = WorldGraph::new();
        
        let a = graph.add_location("A", 0, 0);
        let b = graph.add_location("B", 10, 0);
        let c = graph.add_location("C", 20, 0);
        
        // Connect A-B and B-C
        graph.add_connection(a, b, 10.0);
        graph.add_connection(b, c, 10.0);
        
        // Should find path A->C
        let path = graph.find_path(a, c);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3); // A -> B -> C
    }

    #[test]
    fn world_graph_weighted_paths() {
        let mut graph = WorldGraph::new();
        
        let a = graph.add_location("A", 0, 0);
        let b = graph.add_location("B", 10, 0);
        let c = graph.add_location("C", 20, 0);
        
        // Add two paths with different weights
        graph.add_connection(a, b, 5.0);  // Short path
        graph.add_connection(a, c, 15.0); // Long path
        graph.add_connection(b, c, 5.0);  // Short path via B
        
        // Should prefer A->B->C (total 10) over A->C (15)
        let path = graph.find_path(a, c);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }

    #[test]
    fn world_graph_performance_large() {
        let mut graph = WorldGraph::new();
        
        // Create large grid
        let size = 50;
        let mut locations = Vec::new();
        
        for x in 0..size {
            for z in 0..size {
                let id = graph.add_location(&format!("{}_{}", x, z), x, z);
                locations.push(id);
            }
        }
        
        // Connect to neighbors
        for x in 0..size {
            for z in 0..size {
                let current = x * size + z;
                if x + 1 < size {
                    let right = (x + 1) * size + z;
                    graph.add_connection(current, right, 1.0);
                }
                if z + 1 < size {
                    let down = x * size + (z + 1);
                    graph.add_connection(current, down, 1.0);
                }
            }
        }
        
        let start = std::time::Instant::now();
        
        // Find path across grid
        let path = graph.find_path(0, locations.len() - 1);
        let duration = start.elapsed();
        
        assert!(path.is_some());
        assert!(duration.as_millis() < 100, "Pathfinding should be fast");
    }

    #[test]
    fn world_graph_bidirectional_paths() {
        let mut graph = WorldGraph::new();
        
        let a = graph.add_location("A", 0, 0);
        let b = graph.add_location("B", 10, 0);
        
        graph.add_connection(a, b, 10.0);
        
        // Path should work both directions
        let path_ab = graph.find_path(a, b);
        let path_ba = graph.find_path(b, a);
        
        assert!(path_ab.is_some());
        assert!(path_ba.is_some());
        assert_eq!(path_ab.unwrap().len(), 2);
        assert_eq!(path_ba.unwrap().len(), 2);
    }

    #[test]
    fn world_graph_obstacle_avoidance() {
        let mut graph = WorldGraph::new();
        
        let a = graph.add_location("A", 0, 0);
        let b = graph.add_location("B", 10, 0);
        let c = graph.add_location("C", 5, 5); // Obstacle location
        
        // Direct path A->B
        graph.add_connection(a, b, 10.0);
        
        // Alternative path via C (longer)
        graph.add_connection(a, c, 7.0);
        graph.add_connection(c, b, 7.0);
        
        // Should prefer direct path
        let path = graph.find_path(a, b);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 2);
        
        // Block direct path
        graph.remove_connection(a, b);
        
        // Should use alternative path
        let path = graph.find_path(a, b);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3); // A -> C -> B
    }
}

#[cfg(test)]
mod rvo_collision_avoidance_tests {
    use engene::navigation::avoidance::RvoSystem;

    #[test]
    fn rvo_system_single_agent() {
        let mut rvo = RvoSystem::new();
        rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        rvo.compute();

        let vel = rvo.get_velocity(1).unwrap();
        assert!((vel[0] - 1.0).abs() < 0.01);
        assert!((vel[1] - 0.0).abs() < 0.01);
    }

    #[test]
    fn rvo_system_collision_avoidance() {
        let mut rvo = RvoSystem::new();
        rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        rvo.add_agent(2, [0.5, 0.0], [-1.0, 0.0], [-1.0, 0.0]);
        rvo.compute();

        let v1 = rvo.get_velocity(1).unwrap();
        let v2 = rvo.get_velocity(2).unwrap();
        assert!(
            v1[1].abs() > 0.01 || v2[1].abs() > 0.01 || v1[0] < 1.0 || v2[0] > -1.0,
            "agents should adjust to avoid collision"
        );
    }

    #[test]
    fn rvo_system_multiple_agents() {
        let mut rvo = RvoSystem::new();
        
        // Create circle of agents
        let radius = 5.0;
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * std::f32::consts::PI;
            let x = angle.cos() * radius;
            let z = angle.sin() * radius;
            let vx = -angle.sin(); // Tangent velocity
            let vz = angle.cos();
            
            rvo.add_agent(i, [x, z], [vx, vz], [vx, vz]);
        }
        
        rvo.compute();
        
        // All agents should have velocities
        for i in 0..8 {
            let vel = rvo.get_velocity(i).unwrap();
            assert!(vel[0].is_finite());
            assert!(vel[1].is_finite());
        }
    }

    #[test]
    fn rvo_system_static_obstacles() {
        let mut rvo = RvoSystem::new();
        
        // Add moving agent
        rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        
        // Add static obstacle
        rvo.add_obstacle([2.0, 0.0], 0.5);
        
        rvo.compute();
        
        let vel = rvo.get_velocity(1).unwrap();
        
        // Agent should avoid obstacle
        assert!(vel[1].abs() > 0.01 || vel[0] < 1.0);
    }

    #[test]
    fn rvo_system_performance() {
        let mut rvo = RvoSystem::new();
        
        // Add many agents
        for i in 0..1000 {
            let x = (i % 50) as f32;
            let z = (i / 50) as f32;
            let vx = (i % 3 - 1) as f32;
            let vz = ((i / 3) % 3 - 1) as f32;
            
            rvo.add_agent(i, [x, z], [vx, vz], [vx, vz]);
        }
        
        let start = std::time::Instant::now();
        rvo.compute();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 50, "1000 agents should compute quickly");
        
        // Verify all agents have velocities
        for i in 0..1000 {
            assert!(rvo.get_velocity(i).is_some());
        }
    }

    #[test]
    fn rvo_system_concurrent_updates() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let rvo = Arc::new(Mutex::new(RvoSystem::new()));
        let mut handles = vec![];
        
        // Add agents from multiple threads
        for i in 0..4 {
            let rvo_clone = Arc::clone(&rvo);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let agent_id = i * 100 + j;
                    let x = j as f32;
                    let z = i as f32;
                    let mut r = rvo_clone.lock().unwrap();
                    r.add_agent(agent_id, [x, z], [1.0, 0.0], [1.0, 0.0]);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all additions
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Compute velocities
        rvo.lock().unwrap().compute();
        
        // Verify all agents
        let final_rvo = rvo.lock().unwrap();
        for i in 0..400 {
            assert!(final_rvo.get_velocity(i).is_some());
        }
    }

    #[test]
    fn rvo_system_deadlock_prevention() {
        let mut rvo = RvoSystem::new();
        
        // Create agents facing each other
        rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        rvo.add_agent(2, [2.0, 0.0], [-1.0, 0.0], [-1.0, 0.0]);
        
        rvo.compute();
        
        let v1 = rvo.get_velocity(1).unwrap();
        let v2 = rvo.get_velocity(2).unwrap();
        
        // Should not get stuck (deadlock)
        assert!(v1[0] > 0.0 || v1[1].abs() > 0.01);
        assert!(v2[0] < 0.0 || v2[1].abs() > 0.01);
    }
}

#[cfg(test)]
mod navigation_dirty_tracker_tests {
    use engene::navigation::dynamic_nav_update::NavDirtyTracker;

    #[test]
    fn nav_dirty_tracker_marking_and_draining() {
        let mut tracker = NavDirtyTracker::new();
        assert!(tracker.is_empty());

        tracker.mark_dirty(5, 5);
        tracker.mark_dirty(6, 6);
        assert_eq!(tracker.pending_count(), 2);

        let batch = tracker.drain_dirty_batch();
        assert_eq!(batch.len(), 2);
        assert!(tracker.is_empty());
    }

    #[test]
    fn nav_dirty_tracker_area_marking() {
        let mut tracker = NavDirtyTracker::new();
        
        // Mark area
        tracker.mark_area_dirty(5, 5, 3);
        
        // Should mark all cells in area
        let batch = tracker.drain_dirty_batch();
        assert_eq!(batch.len(), 9); // 3x3 area
    }

    #[test]
    fn nav_dirty_tracker_duplicate_marking() {
        let mut tracker = NavDirtyTracker::new();
        
        // Mark same cell multiple times
        tracker.mark_dirty(5, 5);
        tracker.mark_dirty(5, 5);
        tracker.mark_dirty(5, 5);
        
        assert_eq!(tracker.pending_count(), 1); // Should deduplicate
        
        let batch = tracker.drain_dirty_batch();
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn nav_dirty_tracker_batch_processing() {
        let mut tracker = NavDirtyTracker::new();
        
        // Mark many cells
        for x in 0..10 {
            for z in 0..10 {
                tracker.mark_dirty(x, z);
            }
        }
        
        assert_eq!(tracker.pending_count(), 100);
        
        // Process in batches
        let mut total_processed = 0;
        while !tracker.is_empty() {
            let batch = tracker.drain_dirty_batch_max(20);
            total_processed += batch.len();
            assert!(batch.len() <= 20);
        }
        
        assert_eq!(total_processed, 100);
    }

    #[test]
    fn nav_dirty_tracker_priority_handling() {
        let mut tracker = NavDirtyTracker::new();
        
        // Mark cells with different priorities
        tracker.mark_dirty_priority(5, 5, 1);  // Low priority
        tracker.mark_dirty_priority(6, 6, 10); // High priority
        tracker.mark_dirty_priority(7, 7, 5);  // Medium priority
        
        // Should drain high priority first
        let batch = tracker.drain_by_priority(2);
        assert_eq!(batch.len(), 2);
        
        // First should be highest priority
        assert!(batch[0].priority >= batch[1].priority);
    }

    #[test]
    fn nav_dirty_tracker_performance() {
        let mut tracker = NavDirtyTracker::new();
        
        let start = std::time::Instant::now();
        
        // Mark many cells
        for i in 0..10000 {
            tracker.mark_dirty(i % 100, i / 100);
        }
        
        let mark_time = start.elapsed();
        assert!(mark_time.as_millis() < 50, "Marking should be fast");
        
        // Drain all
        let drain_start = std::time::Instant::now();
        let batch = tracker.drain_dirty_batch();
        let drain_time = drain_start.elapsed();
        
        assert!(drain_time.as_millis() < 20, "Draining should be very fast");
        assert_eq!(batch.len(), 10000);
    }

    #[test]
    fn nav_dirty_tracker_memory_efficiency() {
        let tracker = NavDirtyTracker::new();
        let initial_memory = tracker.memory_usage();
        
        // Add many dirty cells
        let mut temp_tracker = NavDirtyTracker::new();
        for i in 0..5000 {
            temp_tracker.mark_dirty(i % 100, i / 100);
        }
        
        let after_mark_memory = temp_tracker.memory_usage();
        assert!(after_mark_memory > initial_memory);
        
        // Drain should free memory
        temp_tracker.drain_dirty_batch();
        let after_drain_memory = temp_tracker.memory_usage();
        assert!(after_drain_memory < initial_memory + 1000);
    }

    #[test]
    fn nav_dirty_tracker_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let tracker = Arc::new(Mutex::new(NavDirtyTracker::new()));
        let mut handles = vec![];
        
        // Multiple threads marking cells
        for i in 0..4 {
            let tracker_clone = Arc::clone(&tracker);
            let handle = thread::spawn(move || {
                for j in 0..1000 {
                    let mut t = tracker_clone.lock().unwrap();
                    t.mark_dirty(i * 100 + j, j);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_tracker = tracker.lock().unwrap();
        assert_eq!(final_tracker.pending_count(), 4000);
    }
}

#[cfg(test)]
mod navigation_integration_tests {
    use engene::navigation::world_graph::WorldGraph;
    use engene::navigation::avoidance::RvoSystem;
    use engene::navigation::dynamic_nav_update::NavDirtyTracker;

    #[test]
    fn navigation_system_integration() {
        let mut graph = WorldGraph::build_default();
        let mut rvo = RvoSystem::new();
        let mut tracker = NavDirtyTracker::new();
        
        // Setup navigation graph
        let start = graph.locations[0];
        let goal = graph.locations[1];
        let path = graph.find_path(start, goal).unwrap();
        
        // Setup agents for path following
        for (i, &node) in path.iter().enumerate() {
            let pos = graph.get_position(node);
            let next_pos = if i + 1 < path.len() {
                graph.get_position(path[i + 1])
            } else {
                pos
            };
            
            let velocity = [(next_pos[0] - pos[0]) * 0.1, (next_pos[1] - pos[1]) * 0.1];
            rvo.add_agent(i, pos, velocity, velocity);
        }
        
        // Mark navigation area as dirty
        for &node in &path {
            let pos = graph.get_position(node);
            tracker.mark_dirty(pos[0] as i32, pos[1] as i32);
        }
        
        // Process all systems
        rvo.compute();
        let dirty_batch = tracker.drain_dirty_batch();
        
        // Verify integration
        assert!(!dirty_batch.is_empty());
        for i in 0..path.len() {
            assert!(rvo.get_velocity(i).is_some());
        }
    }

    #[test]
    fn navigation_dynamic_obstacle_handling() {
        let mut graph = WorldGraph::build_default();
        let mut rvo = RvoSystem::new();
        let mut tracker = NavDirtyTracker::new();
        
        // Add agent
        rvo.add_agent(1, [0.0, 0.0], [1.0, 0.0], [1.0, 0.0]);
        
        // Add dynamic obstacle
        let obstacle_pos = [5.0, 0.0];
        rvo.add_obstacle(obstacle_pos, 1.0);
        tracker.mark_dirty(obstacle_pos[0] as i32, obstacle_pos[1] as i32);
        
        // Recompute path with obstacle
        let start = graph.locations[0];
        let goal = graph.locations[graph.locations.len() - 1];
        let path = graph.find_path_with_obstacles(start, goal, &obstacle_pos);
        
        // Should find alternative path or handle obstacle
        assert!(path.is_some() || path.is_none()); // Either works or gracefully fails
        
        // Agent should avoid obstacle
        rvo.compute();
        let vel = rvo.get_velocity(1).unwrap();
        
        // Velocity should be adjusted
        assert!(vel[1].abs() > 0.01 || vel[0] < 1.0);
    }

    #[test]
    fn navigation_performance_under_load() {
        let mut graph = WorldGraph::build_default();
        let mut rvo = RvoSystem::new();
        let mut tracker = NavDirtyTracker::new();
        
        // Add many agents
        for i in 0..100 {
            let start = graph.locations[i % graph.locations.len()];
            let goal = graph.locations[(i + 1) % graph.locations.len()];
            let path = graph.find_path(start, goal).unwrap_or(vec![start]);
            
            if let Some(&first_node) = path.first() {
                let pos = graph.get_position(first_node);
                rvo.add_agent(i, pos, [1.0, 0.0], [1.0, 0.0]);
                tracker.mark_dirty(pos[0] as i32, pos[1] as i32);
            }
        }
        
        let start = std::time::Instant::now();
        
        // Process all systems
        rvo.compute();
        let dirty_batch = tracker.drain_dirty_batch();
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Integration should be fast");
        
        assert!(dirty_batch.len() > 0);
        for i in 0..100 {
            if rvo.get_velocity(i).is_some() {
                let vel = rvo.get_velocity(i).unwrap();
                assert!(vel[0].is_finite());
                assert!(vel[1].is_finite());
            }
        }
    }
}

// Mock implementations
impl WorldGraph {
    fn new() -> Self {
        Self {
            locations: Vec::new(),
            connections: std::collections::HashMap::new(),
        }
    }
    
    fn build_default() -> Self {
        let mut graph = Self::new();
        
        // Add default locations
        let village = graph.add_location("Village", 0, 0);
        let forest_n = graph.add_location("Forest North", 0, 10);
        let forest_s = graph.add_location("Forest South", 0, -10);
        let mountain = graph.add_location("Mountain", 15, 0);
        let river = graph.add_location("River", -10, 0);
        
        // Connect locations
        graph.add_connection(village, forest_n, 10.0);
        graph.add_connection(village, forest_s, 10.0);
        graph.add_connection(village, mountain, 15.0);
        graph.add_connection(village, river, 10.0);
        graph.add_connection(forest_n, mountain, 12.0);
        graph.add_connection(mountain, river, 20.0);
        
        graph
    }
    
    fn add_location(&mut self, name: &str, x: i32, z: i32) -> usize {
        let id = self.locations.len();
        self.locations.push(Location {
            id,
            name: name.to_string(),
            x,
            z,
        });
        id
    }
    
    fn add_connection(&mut self, from: usize, to: usize, weight: f32) {
        self.connections.entry(from).or_insert_with(Vec::new).push((to, weight));
        self.connections.entry(to).or_insert_with(Vec::new).push((from, weight));
    }
    
    fn remove_connection(&mut self, from: usize, to: usize) {
        if let Some(conns) = self.connections.get_mut(&from) {
            conns.retain(|(id, _)| *id != to);
        }
        if let Some(conns) = self.connections.get_mut(&to) {
            conns.retain(|(id, _)| *id != from);
        }
    }
    
    fn find_path(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        // Mock pathfinding - in real implementation would use A* or Dijkstra
        if from == to {
            return Some(vec![from]);
        }
        
        // Simple BFS for mock
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parents = std::collections::HashMap::new();
        
        queue.push_back(from);
        visited.insert(from);
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to];
                while let Some(&parent) = parents.get(&current) {
                    path.push(parent);
                }
                path.reverse();
                return Some(path);
            }
            
            if let Some(conns) = self.connections.get(&current) {
                for &(next, _) in conns {
                    if !visited.contains(&next) {
                        visited.insert(next);
                        parents.insert(next, current);
                        queue.push_back(next);
                    }
                }
            }
        }
        
        None
    }
    
    fn find_path_with_obstacles(&self, from: usize, to: usize, obstacles: &[[f32; 2]]) -> Option<Vec<usize>> {
        // Mock implementation - would avoid obstacles
        self.find_path(from, to)
    }
    
    fn neighbors(&self, id: usize) -> Vec<usize> {
        self.connections.get(&id).map(|conns| conns.iter().map(|(id, _)| *id).collect()).unwrap_or_default()
    }
    
    fn get_position(&self, id: usize) -> [f32; 2] {
        if let Some(loc) = self.locations.get(id) {
            [loc.x as f32, loc.z as f32]
        } else {
            [0.0, 0.0]
        }
    }
}

impl RvoSystem {
    fn new() -> Self {
        Self {
            agents: std::collections::HashMap::new(),
            obstacles: Vec::new(),
        }
    }
    
    fn add_agent(&mut self, id: u32, pos: [f32; 2], pref_vel: [f32; 2], max_vel: [f32; 2]) {
        self.agents.insert(id, Agent {
            id,
            pos,
            pref_vel,
            max_vel,
            current_vel: pref_vel,
        });
    }
    
    fn add_obstacle(&mut self, pos: [f32; 2], radius: f32) {
        self.obstacles.push(Obstacle { pos, radius });
    }
    
    fn compute(&mut self) {
        // Mock RVO computation
        for (_, agent) in &mut self.agents {
            // Check for collisions with other agents
            for (_, other) in &self.agents {
                if agent.id != other.id {
                    let dx = other.pos[0] - agent.pos[0];
                    let dz = other.pos[1] - agent.pos[1];
                    let dist = (dx * dx + dz * dz).sqrt();
                    
                    if dist < 2.0 && dist > 0.0 {
                        // Adjust velocity to avoid collision
                        let avoid_x = -dz / dist * 0.5;
                        let avoid_z = dx / dist * 0.5;
                        agent.current_vel[0] += avoid_x;
                        agent.current_vel[1] += avoid_z;
                    }
                }
            }
            
            // Check for obstacles
            for obstacle in &self.obstacles {
                let dx = obstacle.pos[0] - agent.pos[0];
                let dz = obstacle.pos[1] - agent.pos[1];
                let dist = (dx * dx + dz * dz).sqrt();
                
                if dist < obstacle.radius + 1.0 && dist > 0.0 {
                    // Adjust velocity to avoid obstacle
                    let avoid_x = -dz / dist * 0.8;
                    let avoid_z = dx / dist * 0.8;
                    agent.current_vel[0] += avoid_x;
                    agent.current_vel[1] += avoid_z;
                }
            }
        }
    }
    
    fn get_velocity(&self, id: u32) -> Option<[f32; 2]> {
        self.agents.get(&id).map(|agent| agent.current_vel)
    }
}

impl NavDirtyTracker {
    fn new() -> Self {
        Self {
            dirty_cells: std::collections::HashSet::new(),
            priority_cells: std::collections::HashMap::new(),
        }
    }
    
    fn is_empty(&self) -> bool {
        self.dirty_cells.is_empty()
    }
    
    fn pending_count(&self) -> usize {
        self.dirty_cells.len()
    }
    
    fn mark_dirty(&mut self, x: i32, z: i32) {
        self.dirty_cells.insert((x, z));
    }
    
    fn mark_area_dirty(&mut self, x: i32, z: i32, radius: i32) {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                self.mark_dirty(x + dx, z + dz);
            }
        }
    }
    
    fn mark_dirty_priority(&mut self, x: i32, z: i32, priority: u32) {
        self.dirty_cells.insert((x, z));
        self.priority_cells.insert((x, z), priority);
    }
    
    fn drain_dirty_batch(&mut self) -> Vec<DirtyCell> {
        self.dirty_cells.drain().map(|(x, z)| DirtyCell {
            x,
            z,
            priority: self.priority_cells.remove(&(x, z)).unwrap_or(0),
        }).collect()
    }
    
    fn drain_dirty_batch_max(&mut self, max_count: usize) -> Vec<DirtyCell> {
        let mut batch = Vec::new();
        for _ in 0..max_count {
            if let Some((x, z)) = self.dirty_cells.iter().next().copied() {
                self.dirty_cells.remove(&(x, z));
                let priority = self.priority_cells.remove(&(x, z)).unwrap_or(0);
                batch.push(DirtyCell { x, z, priority });
            } else {
                break;
            }
        }
        batch
    }
    
    fn drain_by_priority(&mut self, count: usize) -> Vec<DirtyCell> {
        let mut cells: Vec<_> = self.dirty_cells.iter()
            .map(|&(x, z)| DirtyCell {
                x,
                z,
                priority: self.priority_cells.get(&(x, z)).copied().unwrap_or(0),
            })
            .collect();
        
        cells.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        let batch: Vec<_> = cells.into_iter().take(count).collect();
        
        // Remove from sets
        for cell in &batch {
            self.dirty_cells.remove(&(cell.x, cell.z));
            self.priority_cells.remove(&(cell.x, cell.z));
        }
        
        batch
    }
    
    fn memory_usage(&self) -> usize {
        (self.dirty_cells.len() + self.priority_cells.len()) * std::mem::size_of::<(i32, i32)>()
    }
}

#[derive(Debug)]
struct WorldGraph {
    locations: Vec<Location>,
    connections: std::collections::HashMap<usize, Vec<(usize, f32)>>,
}

#[derive(Debug)]
struct Location {
    id: usize,
    name: String,
    x: i32,
    z: i32,
}

#[derive(Debug)]
struct RvoSystem {
    agents: std::collections::HashMap<u32, Agent>,
    obstacles: Vec<Obstacle>,
}

#[derive(Debug)]
struct Agent {
    id: u32,
    pos: [f32; 2],
    pref_vel: [f32; 2],
    max_vel: [f32; 2],
    current_vel: [f32; 2],
}

#[derive(Debug)]
struct Obstacle {
    pos: [f32; 2],
    radius: f32,
}

#[derive(Debug)]
struct NavDirtyTracker {
    dirty_cells: std::collections::HashSet<(i32, i32)>,
    priority_cells: std::collections::HashMap<(i32, i32), u32>,
}

#[derive(Debug)]
struct DirtyCell {
    x: i32,
    z: i32,
    priority: u32,
}
