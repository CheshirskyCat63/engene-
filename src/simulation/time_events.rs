#[derive(Clone, Debug)]
pub struct NewDay {
    pub day: u32,
    pub month: u32,
}

#[derive(Clone, Debug)]
pub struct NewMonth(pub u32);
