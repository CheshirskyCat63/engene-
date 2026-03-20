use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Fence {
    signaled: Arc<AtomicBool>,
}

impl Fence {
    pub fn new() -> Self {
        Self {
            signaled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn signal(&self) {
        self.signaled.store(true, Ordering::Release);
    }

    pub fn is_signaled(&self) -> bool {
        self.signaled.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.signaled.store(false, Ordering::Release);
    }

    pub fn wait_blocking(&self) {
        while !self.is_signaled() {
            std::hint::spin_loop();
        }
    }
}

impl Default for Fence {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Fence {
    fn clone(&self) -> Self {
        Self {
            signaled: Arc::clone(&self.signaled),
        }
    }
}
