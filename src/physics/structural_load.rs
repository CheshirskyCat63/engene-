use crate::physics::building::StructuralSection;

const MAX_CASCADE_ITERATIONS: u32 = 8;

pub fn redistribute_loads(sections: &mut [StructuralSection], collapsed_id: u32) -> Vec<u32> {
    let mut newly_collapsed = Vec::new();

    let collapsed_load: f32 = {
        let sec = match sections.iter().find(|s| s.id == collapsed_id) {
            Some(s) => s,
            None => return newly_collapsed,
        };
        1.0 - sec.integrity
    };

    let neighbor_data: Vec<(u32, f32)> = sections
        .iter()
        .find(|s| s.id == collapsed_id)
        .map(|s| {
            s.neighbors
                .iter()
                .map(|n| (n.section_id, n.load_transfer))
                .collect()
        })
        .unwrap_or_default();

    for (nid, transfer) in neighbor_data {
        if let Some(neighbor) = sections.iter_mut().find(|s| s.id == nid) {
            let extra_load = collapsed_load * transfer;
            neighbor.integrity -= extra_load * 0.1;
            if neighbor.integrity <= 0.0 {
                neighbor.integrity = 0.0;
                newly_collapsed.push(nid);
            }
        }
    }

    newly_collapsed
}

pub fn cascade_collapse(sections: &mut [StructuralSection], initial_collapsed: u32) -> Vec<u32> {
    let mut all_collapsed = vec![initial_collapsed];
    let mut queue = vec![initial_collapsed];
    let mut iterations = 0;

    while let Some(sid) = queue.pop() {
        if iterations >= MAX_CASCADE_ITERATIONS {
            break;
        }
        iterations += 1;

        let newly = redistribute_loads(sections, sid);
        for nid in newly {
            if !all_collapsed.contains(&nid) {
                all_collapsed.push(nid);
                queue.push(nid);
            }
        }
    }

    all_collapsed
}
