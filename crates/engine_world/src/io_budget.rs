use super::chunk_package::{BundleKind, StreamPriority};
use super::streaming::ChunkCoord;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
pub struct IoBudget {
    pub read_bytes_per_frame: u64,
    pub decompress_bytes_per_frame: u64,
    pub upload_bytes_per_frame: u64,
}

impl Default for IoBudget {
    fn default() -> Self {
        Self {
            read_bytes_per_frame: 2 * 1024 * 1024,
            decompress_bytes_per_frame: 4 * 1024 * 1024,
            upload_bytes_per_frame: 1 * 1024 * 1024,
        }
    }
}

impl IoBudget {
    pub fn low_spec() -> Self {
        Self {
            read_bytes_per_frame: 512 * 1024,
            decompress_bytes_per_frame: 1024 * 1024,
            upload_bytes_per_frame: 512 * 1024,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreamRequest {
    pub coord: ChunkCoord,
    pub bundle_kind: BundleKind,
    pub priority: StreamPriority,
    pub size_bytes: u64,
    pub compressed_bytes: u64,
    pub distance_sq: f32,
}

impl Eq for StreamRequest {}
impl PartialEq for StreamRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.distance_sq.to_bits() == other.distance_sq.to_bits()
    }
}

impl PartialOrd for StreamRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StreamRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.priority as u8)
            .cmp(&(self.priority as u8))
            .then_with(|| other.distance_sq.to_bits().cmp(&self.distance_sq.to_bits()))
    }
}

#[derive(Debug)]
pub struct FrameStreamResult {
    pub loaded: Vec<(ChunkCoord, BundleKind)>,
    pub deferred: usize,
    pub read_bytes_used: u64,
    pub decompress_bytes_used: u64,
}

pub struct BudgetedStreamer {
    budget: IoBudget,
    queue: BinaryHeap<StreamRequest>,
}

impl BudgetedStreamer {
    pub fn new(budget: IoBudget) -> Self {
        Self {
            budget,
            queue: BinaryHeap::new(),
        }
    }

    pub fn enqueue(&mut self, request: StreamRequest) {
        self.queue.push(request);
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    pub fn process_frame(&mut self) -> FrameStreamResult {
        let mut read_remaining = self.budget.read_bytes_per_frame;
        let mut decompress_remaining = self.budget.decompress_bytes_per_frame;
        let mut loaded = Vec::new();
        let mut deferred = Vec::new();

        while let Some(req) = self.queue.pop() {
            if req.compressed_bytes <= read_remaining && req.size_bytes <= decompress_remaining {
                read_remaining -= req.compressed_bytes;
                decompress_remaining -= req.size_bytes;
                loaded.push((req.coord, req.bundle_kind));
            } else {
                deferred.push(req);
            }
        }

        let deferred_count = deferred.len();
        for req in deferred {
            self.queue.push(req);
        }

        FrameStreamResult {
            loaded,
            deferred: deferred_count,
            read_bytes_used: self.budget.read_bytes_per_frame - read_remaining,
            decompress_bytes_used: self.budget.decompress_bytes_per_frame - decompress_remaining,
        }
    }

    pub fn set_budget(&mut self, budget: IoBudget) {
        self.budget = budget;
    }
}
