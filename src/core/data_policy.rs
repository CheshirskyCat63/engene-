#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTemperature {
    Hot,
    Cold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessPattern {
    EveryTick,
    PeriodicHigh,
    PeriodicLow,
    OnDemand,
    Rare,
}

impl AccessPattern {
    pub fn temperature(self) -> DataTemperature {
        match self {
            Self::EveryTick | Self::PeriodicHigh => DataTemperature::Hot,
            Self::PeriodicLow | Self::OnDemand | Self::Rare => DataTemperature::Cold,
        }
    }
}
