pub struct GameTime {
    pub delta: f32,
    pub elapsed: f32,
    pub tick_count: u64,
    pub time_scale: f32,
    pub day: u32,
    pub month: u32,
    pub year: u32,
    seconds_in_day: f32,
}

impl GameTime {
    const SECONDS_PER_DAY: f32 = 120.0;
    const DAYS_PER_MONTH: u32 = 30;

    pub fn new() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            tick_count: 0,
            time_scale: 1.0,
            day: 1,
            month: 1,
            year: 1,
            seconds_in_day: 0.0,
        }
    }

    pub fn advance(&mut self, real_delta: f32) -> TimeEvents {
        self.delta = real_delta * self.time_scale;
        self.elapsed += self.delta;
        self.tick_count += 1;

        let mut events = TimeEvents {
            new_day: false,
            new_month: false,
        };

        self.seconds_in_day += self.delta;
        if self.seconds_in_day >= Self::SECONDS_PER_DAY {
            self.seconds_in_day -= Self::SECONDS_PER_DAY;
            self.day += 1;
            events.new_day = true;

            if self.day > Self::DAYS_PER_MONTH {
                self.day = 1;
                self.month += 1;
                events.new_month = true;
            }
        }

        events
    }

    pub fn day_progress(&self) -> f32 {
        self.seconds_in_day / Self::SECONDS_PER_DAY
    }
}

pub struct TimeEvents {
    pub new_day: bool,
    pub new_month: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn food_regen_mult(&self) -> f32 {
        match self {
            Self::Spring => 1.5,
            Self::Summer => 1.2,
            Self::Autumn => 0.8,
            Self::Winter => 0.3,
        }
    }

    pub fn hunger_drain_mult(&self) -> f32 {
        match self {
            Self::Spring => 1.0,
            Self::Summer => 1.0,
            Self::Autumn => 1.1,
            Self::Winter => 1.4,
        }
    }

    pub fn breeding_mult(&self) -> f32 {
        match self {
            Self::Spring => 1.5,
            Self::Summer => 1.0,
            Self::Autumn => 0.5,
            Self::Winter => 0.0,
        }
    }

    pub fn danger_mult(&self) -> f32 {
        match self {
            Self::Spring => 1.0,
            Self::Summer => 1.0,
            Self::Autumn => 1.1,
            Self::Winter => 1.5,
        }
    }
}

impl GameTime {
    pub fn season(&self) -> Season {
        match ((self.month.saturating_sub(1)) % 12) / 3 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

impl std::fmt::Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spring => write!(f, "Spring"),
            Self::Summer => write!(f, "Summer"),
            Self::Autumn => write!(f, "Autumn"),
            Self::Winter => write!(f, "Winter"),
        }
    }
}
