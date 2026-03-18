use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use std::collections::HashMap;

const CLUSTER_SIZE: u32 = 8;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ClusterCoord {
    pub cx: u32,
    pub cy: u32,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BorderNode {
    pub cluster: ClusterCoord,
    pub local_x: u32,
    pub local_y: u32,
}

impl BorderNode {
    pub fn world_pos(&self) -> (f32, f32) {
        let wx =
            (self.cluster.cx * CLUSTER_SIZE + self.local_x) as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let wy =
            (self.cluster.cy * CLUSTER_SIZE + self.local_y) as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        (wx, wy)
    }
}

pub struct HpaGraph {
    pub clusters_w: u32,
    pub clusters_h: u32,
    pub border_nodes: Vec<BorderNode>,
    pub edges: HashMap<usize, Vec<(usize, f32)>>,
}

impl HpaGraph {
    pub fn build() -> Self {
        let cw = (GRID_SIZE + CLUSTER_SIZE - 1) / CLUSTER_SIZE;
        let ch = cw;
        let mut border_nodes = Vec::new();
        let mut edges: HashMap<usize, Vec<(usize, f32)>> = HashMap::new();

        for cy in 0..ch {
            for cx in 0..cw {
                let cluster = ClusterCoord { cx, cy };
                if cx + 1 < cw {
                    let right_cluster = ClusterCoord { cx: cx + 1, cy };
                    let border_x = CLUSTER_SIZE - 1;
                    for ly in 0..CLUSTER_SIZE.min(GRID_SIZE - cy * CLUSTER_SIZE) {
                        let a = border_nodes.len();
                        border_nodes.push(BorderNode {
                            cluster,
                            local_x: border_x,
                            local_y: ly,
                        });
                        let b = border_nodes.len();
                        border_nodes.push(BorderNode {
                            cluster: right_cluster,
                            local_x: 0,
                            local_y: ly,
                        });
                        edges.entry(a).or_default().push((b, CELL_SIZE));
                        edges.entry(b).or_default().push((a, CELL_SIZE));
                    }
                }
                if cy + 1 < ch {
                    let bottom_cluster = ClusterCoord { cx, cy: cy + 1 };
                    let border_y = CLUSTER_SIZE - 1;
                    for lx in 0..CLUSTER_SIZE.min(GRID_SIZE - cx * CLUSTER_SIZE) {
                        let a = border_nodes.len();
                        border_nodes.push(BorderNode {
                            cluster,
                            local_x: lx,
                            local_y: border_y,
                        });
                        let b = border_nodes.len();
                        border_nodes.push(BorderNode {
                            cluster: bottom_cluster,
                            local_x: lx,
                            local_y: 0,
                        });
                        edges.entry(a).or_default().push((b, CELL_SIZE));
                        edges.entry(b).or_default().push((a, CELL_SIZE));
                    }
                }
            }
        }

        Self {
            clusters_w: cw,
            clusters_h: ch,
            border_nodes,
            edges,
        }
    }

    pub fn find_abstract_path(&self, start: (f32, f32), goal: (f32, f32)) -> Vec<(f32, f32)> {
        let start_node = self.nearest_node(start);
        let goal_node = self.nearest_node(goal);
        if start_node == goal_node {
            return vec![goal];
        }

        let result = pathfinding::directed::astar::astar(
            &start_node,
            |&n| {
                self.edges.get(&n).map_or(Vec::new(), |neighbors| {
                    neighbors
                        .iter()
                        .map(|&(next, cost)| (next, (cost * 100.0) as u32))
                        .collect()
                })
            },
            |&n| {
                let (nx, ny) = self.border_nodes[n].world_pos();
                let (gx, gy) = self.border_nodes[goal_node].world_pos();
                let dx = nx - gx;
                let dy = ny - gy;
                ((dx * dx + dy * dy).sqrt() * 100.0) as u32
            },
            |&n| n == goal_node,
        );

        match result {
            Some((path, _cost)) => path
                .iter()
                .map(|&n| self.border_nodes[n].world_pos())
                .collect(),
            None => vec![goal],
        }
    }

    fn nearest_node(&self, pos: (f32, f32)) -> usize {
        let (px, py) = pos;
        let mut best = 0;
        let mut best_dist = f32::MAX;
        for (i, node) in self.border_nodes.iter().enumerate() {
            let (nx, ny) = node.world_pos();
            let d = (px - nx) * (px - nx) + (py - ny) * (py - ny);
            if d < best_dist {
                best_dist = d;
                best = i;
            }
        }
        best
    }
}
