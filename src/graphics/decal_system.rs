/// Decal System 2.0 (Phase B.5)
/// Classified decals with semantic types, weather interaction, and priority.

/// Semantic decal classification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecalClass {
    Impact,
    Crack,
    Leak,
    DirtAccumulation,
    BulletGraze,
    WetFootprint,
    DragMark,
    BloodTransfer,
    BurnSpread,
    ScorchMark,
    MudSplash,
}

impl DecalClass {
    /// Base lifetime in seconds for this decal class
    pub fn base_lifetime(&self) -> f32 {
        match self {
            Self::Impact => 300.0,
            Self::Crack => f32::INFINITY,
            Self::Leak => 120.0,
            Self::DirtAccumulation => 600.0,
            Self::BulletGraze => f32::INFINITY,
            Self::WetFootprint => 30.0,
            Self::DragMark => 60.0,
            Self::BloodTransfer => 180.0,
            Self::BurnSpread => f32::INFINITY,
            Self::ScorchMark => f32::INFINITY,
            Self::MudSplash => 90.0,
        }
    }

    /// Priority for eviction (lower = evicted first)
    pub fn priority(&self) -> u8 {
        match self {
            Self::WetFootprint => 1,
            Self::MudSplash => 2,
            Self::DragMark => 3,
            Self::Leak => 4,
            Self::DirtAccumulation => 5,
            Self::BloodTransfer => 6,
            Self::Impact => 7,
            Self::ScorchMark => 8,
            Self::BulletGraze => 9,
            Self::Crack => 10,
            Self::BurnSpread => 10,
        }
    }

    /// Whether rain washes this decal away
    pub fn rain_washable(&self) -> bool {
        matches!(self, Self::WetFootprint | Self::MudSplash | Self::DragMark
            | Self::BloodTransfer | Self::DirtAccumulation | Self::Leak)
    }
}

/// A classified decal instance
#[derive(Clone, Debug)]
pub struct ClassifiedDecal {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub size: f32,
    pub class: DecalClass,
    pub intensity: f32,
    pub age: f32,
    pub lifetime: f32,
}

/// Decal System 2.0 with classification, budget, and weather interaction
pub struct DecalSystemV2 {
    decals: Vec<ClassifiedDecal>,
    max_decals: usize,
}

impl DecalSystemV2 {
    pub fn new(max_decals: usize) -> Self {
        Self {
            decals: Vec::with_capacity(max_decals),
            max_decals,
        }
    }

    /// Add a classified decal
    pub fn add(&mut self, position: [f32; 3], normal: [f32; 3], size: f32, class: DecalClass, intensity: f32) {
        let decal = ClassifiedDecal {
            position,
            normal,
            size,
            class,
            intensity,
            age: 0.0,
            lifetime: class.base_lifetime(),
        };

        if self.decals.len() >= self.max_decals {
            self.evict_lowest_priority();
        }

        self.decals.push(decal);
    }

    /// Update all decals: age, remove expired, apply weather
    pub fn update(&mut self, dt: f32, rain_intensity: f32) {
        for decal in &mut self.decals {
            decal.age += dt;
            if rain_intensity > 0.0 && decal.class.rain_washable() {
                decal.intensity -= rain_intensity * dt * 0.1;
            }
        }

        self.decals.retain(|d| {
            d.age < d.lifetime && d.intensity > 0.01
        });
    }

    fn evict_lowest_priority(&mut self) {
        if let Some(idx) = self.decals.iter().enumerate()
            .min_by_key(|(_, d)| d.class.priority())
            .map(|(i, _)| i)
        {
            self.decals.swap_remove(idx);
        }
    }

    pub fn active_count(&self) -> usize {
        self.decals.len()
    }

    pub fn decals(&self) -> &[ClassifiedDecal] {
        &self.decals
    }

    pub fn count_by_class(&self, class: DecalClass) -> usize {
        self.decals.iter().filter(|d| d.class == class).count()
    }
}
