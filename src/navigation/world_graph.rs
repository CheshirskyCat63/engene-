use std::collections::HashMap;
use pathfinding::directed::astar::astar;

pub type LocationId = u32;

#[derive(Clone, Debug)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub cell_x: u32,
    pub cell_y: u32,
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub from: LocationId,
    pub to: LocationId,
    pub distance: f32,
}

pub struct WorldGraph {
    pub locations: HashMap<LocationId, Location>,
    pub edges: Vec<Edge>,
    next_id: LocationId,
}

impl WorldGraph {
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            edges: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_location(&mut self, name: &str, cell_x: u32, cell_y: u32) -> LocationId {
        let id = self.next_id;
        self.next_id += 1;
        self.locations.insert(
            id,
            Location {
                id,
                name: name.to_string(),
                cell_x,
                cell_y,
            },
        );
        id
    }

    pub fn connect(&mut self, a: LocationId, b: LocationId) {
        let dist = self.distance_between(a, b);
        self.edges.push(Edge { from: a, to: b, distance: dist });
        self.edges.push(Edge { from: b, to: a, distance: dist });
    }

    pub fn neighbors(&self, id: LocationId) -> Vec<LocationId> {
        self.edges
            .iter()
            .filter(|e| e.from == id)
            .map(|e| e.to)
            .collect()
    }

    pub fn distance_between(&self, a: LocationId, b: LocationId) -> f32 {
        let la = match self.locations.get(&a) {
            Some(l) => l,
            None => return f32::MAX,
        };
        let lb = match self.locations.get(&b) {
            Some(l) => l,
            None => return f32::MAX,
        };
        let dx = la.cell_x as f32 - lb.cell_x as f32;
        let dy = la.cell_y as f32 - lb.cell_y as f32;
        (dx * dx + dy * dy).sqrt() * crate::world::cell::CELL_SIZE
    }

    pub fn find_path(&self, from: LocationId, to: LocationId) -> Option<Vec<LocationId>> {
        if from == to {
            return Some(vec![from]);
        }
        if !self.locations.contains_key(&from) || !self.locations.contains_key(&to) {
            return None;
        }
        let result = astar(
            &from,
            |&node| {
                self.edges
                    .iter()
                    .filter(move |e| e.from == node)
                    .map(|e| (e.to, (e.distance * 100.0) as u32))
            },
            |&node| (self.distance_between(node, to) * 100.0) as u32,
            |&node| node == to,
        );
        result.map(|(path, _cost)| path)
    }

    pub fn build_default() -> Self {
        let mut g = Self::new();
        let village = g.add_location("Village", 10, 10);
        let forest_n = g.add_location("Northern Forest", 5, 3);
        let forest_s = g.add_location("Southern Forest", 14, 14);
        let swamp = g.add_location("Swamp", 3, 17);
        let hills = g.add_location("Hills", 16, 5);

        g.connect(village, forest_n);
        g.connect(village, forest_s);
        g.connect(village, hills);
        g.connect(forest_n, swamp);
        g.connect(forest_s, swamp);

        g
    }
}
