use crate::physics::building::StructuralSection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureMode {
    Crack,
    LocalBreak,
    PartialCollapse,
    FullCollapse,
    Hanging,
}

pub struct CollapseResult {
    pub section_id: u32,
    pub failure_mode: FailureMode,
}

pub fn evaluate_failure(section: &StructuralSection) -> Option<CollapseResult> {
    if section.integrity >= 0.9 {
        return None;
    }

    let mode = if section.integrity >= 0.6 {
        FailureMode::Crack
    } else if section.integrity >= 0.3 {
        FailureMode::LocalBreak
    } else if section.integrity >= 0.1 {
        FailureMode::PartialCollapse
    } else {
        FailureMode::FullCollapse
    };

    Some(CollapseResult {
        section_id: section.id,
        failure_mode: mode,
    })
}

pub fn evaluate_hanging(section: &StructuralSection) -> bool {
    let supported_count = section
        .neighbors
        .iter()
        .filter(|n| n.load_transfer > 0.3)
        .count();
    supported_count == 0 && section.integrity > 0.0
}
