#[derive(Clone, Debug)]
pub struct DivergencePoint {
    pub tick: u64,
    pub description: String,
    pub severity: DivergenceSeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DivergenceSeverity {
    Exact,
    Minor,
    Major,
    Critical,
}

pub struct DivergenceDetector {
    points: Vec<DivergencePoint>,
    tolerance: f32,
}

impl DivergenceDetector {
    pub fn new(tolerance: f32) -> Self {
        Self {
            points: Vec::new(),
            tolerance,
        }
    }

    pub fn compare_snapshots(&mut self, tick: u64, expected: &[u8], actual: &[u8]) {
        if expected == actual {
            return;
        }

        let diff_bytes = expected
            .iter()
            .zip(actual.iter())
            .filter(|(a, b)| a != b)
            .count();

        let diff_ratio = if expected.is_empty() {
            1.0
        } else {
            diff_bytes as f32 / expected.len() as f32
        };

        let severity = if diff_bytes == 0 {
            DivergenceSeverity::Exact
        } else if diff_bytes < expected.len() / 100 {
            DivergenceSeverity::Minor
        } else if diff_bytes < expected.len() / 10 {
            DivergenceSeverity::Major
        } else {
            DivergenceSeverity::Critical
        };

        let adjusted_severity =
            if diff_ratio <= self.tolerance && severity != DivergenceSeverity::Critical {
                DivergenceSeverity::Minor
            } else {
                severity
            };

        self.points.push(DivergencePoint {
            tick,
            description: format!(
                "{} bytes differ out of {} (ratio: {:.4})",
                diff_bytes,
                expected.len(),
                diff_ratio
            ),
            severity: adjusted_severity,
        });
    }

    pub fn has_critical(&self) -> bool {
        self.points
            .iter()
            .any(|p| p.severity == DivergenceSeverity::Critical)
    }

    pub fn points(&self) -> &[DivergencePoint] {
        &self.points
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }
}
