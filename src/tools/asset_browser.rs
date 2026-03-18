#[derive(Clone, Debug)]
pub struct AssetEntry {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub size_bytes: u64,
    pub path: String,
    pub cooked: bool,
}

pub struct AssetBrowser {
    assets: Vec<AssetEntry>,
    filter_type: Option<String>,
    filter_text: String,
    selected: Option<u64>,
    sort_by: SortField,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortField {
    Name,
    Type,
    Size,
}

impl AssetBrowser {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            filter_type: None,
            filter_text: String::new(),
            selected: None,
            sort_by: SortField::Name,
        }
    }

    pub fn populate(&mut self, assets: Vec<AssetEntry>) {
        self.assets = assets;
        self.apply_sort();
    }

    pub fn set_filter_type(&mut self, asset_type: Option<String>) {
        self.filter_type = asset_type;
    }

    pub fn set_filter_text(&mut self, text: &str) {
        self.filter_text = text.to_lowercase();
    }

    pub fn set_sort(&mut self, field: SortField) {
        self.sort_by = field;
        self.apply_sort();
    }

    fn apply_sort(&mut self) {
        match self.sort_by {
            SortField::Name => self.assets.sort_by(|a, b| a.name.cmp(&b.name)),
            SortField::Type => self.assets.sort_by(|a, b| a.asset_type.cmp(&b.asset_type)),
            SortField::Size => self.assets.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes)),
        }
    }

    pub fn visible_assets(&self) -> Vec<&AssetEntry> {
        self.assets
            .iter()
            .filter(|a| {
                if let Some(ref t) = self.filter_type {
                    if a.asset_type != *t {
                        return false;
                    }
                }
                if !self.filter_text.is_empty() {
                    if !a.name.to_lowercase().contains(&self.filter_text) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn select(&mut self, id: u64) {
        self.selected = Some(id);
    }

    pub fn selected_asset(&self) -> Option<&AssetEntry> {
        let id = self.selected?;
        self.assets.iter().find(|a| a.id == id)
    }

    pub fn asset_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self
            .assets
            .iter()
            .map(|a| a.asset_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        types.sort();
        types
    }

    pub fn total_size(&self) -> u64 {
        self.assets.iter().map(|a| a.size_bytes).sum()
    }

    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    #[cfg(feature = "debug_ui")]
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Browser");

        ui.horizontal(|ui| {
            ui.label("Search:");
            let mut text = self.filter_text.clone();
            if ui.text_edit_singleline(&mut text).changed() {
                self.set_filter_text(&text);
            }
        });

        ui.separator();
        ui.label(format!(
            "{} assets, {:.1} MB total",
            self.asset_count(),
            self.total_size() as f64 / (1024.0 * 1024.0)
        ));
        ui.separator();

        let visible: Vec<(u64, String)> = self
            .visible_assets()
            .iter()
            .map(|a| {
                (
                    a.id,
                    format!(
                        "[{}] {} ({:.1} KB)",
                        a.asset_type,
                        a.name,
                        a.size_bytes as f64 / 1024.0
                    ),
                )
            })
            .collect();
        for (id, label) in &visible {
            let selected = self.selected == Some(*id);
            if ui.selectable_label(selected, label).clicked() {
                self.selected = Some(*id);
            }
        }
    }
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new()
    }
}
