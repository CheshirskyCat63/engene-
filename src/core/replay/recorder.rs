#[derive(Clone, Debug)]
pub struct ReplayFrame {
    pub tick: u64,
    pub input_snapshot: Vec<u8>,
    pub events_snapshot: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ReplayHeader {
    pub seed: u64,
    pub tick_rate: f32,
    pub version: u32,
    pub frame_count: u64,
}

pub struct ReplayRecorder {
    header: ReplayHeader,
    frames: Vec<ReplayFrame>,
    recording: bool,
}

impl ReplayRecorder {
    pub fn new(seed: u64, tick_rate: f32) -> Self {
        Self {
            header: ReplayHeader {
                seed,
                tick_rate,
                version: 1,
                frame_count: 0,
            },
            frames: Vec::new(),
            recording: false,
        }
    }

    pub fn start(&mut self) { self.recording = true; }
    pub fn stop(&mut self) { self.recording = false; }
    pub fn is_recording(&self) -> bool { self.recording }

    pub fn record_frame(&mut self, tick: u64, inputs: &[u8], events: &[u8]) {
        if !self.recording { return; }
        self.frames.push(ReplayFrame {
            tick,
            input_snapshot: inputs.to_vec(),
            events_snapshot: events.to_vec(),
        });
        self.header.frame_count += 1;
    }

    pub fn header(&self) -> &ReplayHeader { &self.header }
    pub fn frame_count(&self) -> usize { self.frames.len() }

    pub fn get_frame(&self, index: usize) -> Option<&ReplayFrame> {
        self.frames.get(index)
    }

    pub fn frames(&self) -> &[ReplayFrame] {
        &self.frames
    }
}

pub struct ReplayPlayer {
    frames: Vec<ReplayFrame>,
    current_frame: usize,
    playing: bool,
}

impl ReplayPlayer {
    pub fn new(frames: Vec<ReplayFrame>) -> Self {
        Self { frames, current_frame: 0, playing: false }
    }

    pub fn play(&mut self) { self.playing = true; }
    pub fn pause(&mut self) { self.playing = false; }
    pub fn is_playing(&self) -> bool { self.playing }

    pub fn advance(&mut self) -> Option<&ReplayFrame> {
        if !self.playing || self.current_frame >= self.frames.len() {
            return None;
        }
        let frame = &self.frames[self.current_frame];
        self.current_frame += 1;
        Some(frame)
    }

    pub fn seek(&mut self, frame_index: usize) {
        self.current_frame = frame_index.min(self.frames.len());
    }

    pub fn current_frame_index(&self) -> usize { self.current_frame }
    pub fn total_frames(&self) -> usize { self.frames.len() }
    pub fn is_finished(&self) -> bool { self.current_frame >= self.frames.len() }
}
