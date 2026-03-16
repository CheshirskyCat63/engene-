use bitflags::bitflags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DamageClass {
    Ballistic,
    Blunt,
    Explosive,
    Piercing,
    Shear,
    Fragmentation,
    Thermal,
    Hydraulic,
    Erosion,
    Corrosion,
    Fatigue,
}

impl DamageClass {
    pub fn is_instant(self) -> bool {
        matches!(
            self,
            Self::Ballistic
                | Self::Blunt
                | Self::Explosive
                | Self::Piercing
                | Self::Shear
                | Self::Fragmentation
        )
    }

    pub fn is_cumulative(self) -> bool {
        !self.is_instant()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct DamageCapability: u8 {
        const SURFACE    = 0b0000_0001;
        const LAYERED    = 0b0000_0011;
        const STRUCTURAL = 0b0000_0111;
        const ANATOMICAL = 0b0000_1111;
        const THERMAL    = 0b0001_0000;
        const MOISTURE   = 0b0010_0000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResponseOwner {
    SurfaceResolver,
    LayerResolver,
    StructuralResolver,
    BodyResolver,
    ResponseAggregator,
    AudioConsumer,
    AiConsumer,
}
