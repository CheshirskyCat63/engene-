use crate::core::replay::recorder::ReplayHeader;

pub struct ReplayBrowser {
    recordings: Vec<ReplayEntry>,
    selected: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct ReplayEntry {
    pub name: String,
    pub header: ReplayHeader,
    pub path: Option<std::path::PathBuf>,
    pub frame_count: usize,
}

impl ReplayBrowser {
    pub fn new() -> Self {
        Self {
            recordings: Vec::new(),
            selected: None,
        }
    }

    pub fn add_recording(&mut self, name: &str, header: ReplayHeader, frame_count: usize) {
        self.recordings.push(ReplayEntry {
            name: name.to_string(),
            header,
            path: None,
            frame_count,
        });
    }

    pub fn scan_directory(&mut self, dir: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "replay") {
                    let name = path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.recordings.push(ReplayEntry {
                        name,
                        header: ReplayHeader {
                            seed: 0,
                            tick_rate: 60.0,
                            version: 1,
                            frame_count: 0,
                        },
                        path: Some(path),
                        frame_count: 0,
                    });
                }
            }
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.recordings.len() {
            self.selected = Some(index);
        }
    }

    pub fn selected_entry(&self) -> Option<&ReplayEntry> {
        self.selected.and_then(|i| self.recordings.get(i))
    }

    pub fn recording_count(&self) -> usize {
        self.recordings.len()
    }

    pub fn recordings(&self) -> &[ReplayEntry] {
        &self.recordings
    }

    #[cfg(feature = "debug_ui")]
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Replay Browser");
        ui.separator();

        for (i, entry) in self.recordings.iter().enumerate() {
            let selected = self.selected == Some(i);
            let label = format!("{} ({} frames, seed={})", entry.name, entry.frame_count, entry.header.seed);
            if ui.selectable_label(selected, &label).clicked() {
                self.selected = Some(i);
            }
        }

        if let Some(entry) = self.selected_entry() {
            ui.separator();
            ui.label(format!("Tick rate: {:.1}", entry.header.tick_rate));
            ui.label(format!("Version: {}", entry.header.version));
        }
    }
}

impl Default for ReplayBrowser {
    fn default() -> Self { Self::new() }
}
