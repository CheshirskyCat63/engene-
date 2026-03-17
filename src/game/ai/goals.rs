use crate::world::components::Goal;

pub struct ScoredGoal {
    pub goal: Goal,
    pub score: f32,
}

pub fn pick_best(candidates: &[ScoredGoal]) -> Goal {
    candidates
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .map(|sg| sg.goal)
        .unwrap_or(Goal::Rest)
}
