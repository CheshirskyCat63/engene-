use std::collections::VecDeque;

pub struct DebugFrameSnapshot {
    pub tick: u64,
    pub data: Vec<(String, String)>,
}

pub struct FrameRecorder {
    frames: VecDeque<DebugFrameSnapshot>,
    capacity: usize,
    frozen: bool,
    thinning_factor: u32,
    frame_counter: u64,
}

impl FrameRecorder {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(capacity),
            capacity,
            frozen: false,
            thinning_factor: 1,
            frame_counter: 0,
        }
    }

    pub fn record(&mut self, snapshot: DebugFrameSnapshot) {
        if self.frozen { return; }
        self.frame_counter += 1;
        if self.frame_counter % self.thinning_factor as u64 != 0 { return; }
        if self.frames.len() >= self.capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(snapshot);
    }

    pub fn freeze(&mut self) { self.frozen = true; }
    pub fn unfreeze(&mut self) { self.frozen = false; }
    pub fn is_frozen(&self) -> bool { self.frozen }

    pub fn set_thinning(&mut self, factor: u32) {
        self.thinning_factor = factor.max(1);
    }

    pub fn get_frame(&self, index: usize) -> Option<&DebugFrameSnapshot> {
        self.frames.get(index)
    }

    pub fn latest(&self) -> Option<&DebugFrameSnapshot> {
        self.frames.back()
    }

    pub fn len(&self) -> usize { self.frames.len() }
    pub fn is_empty(&self) -> bool { self.frames.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }

    pub fn iter(&self) -> impl Iterator<Item = &DebugFrameSnapshot> {
        self.frames.iter()
    }
}
