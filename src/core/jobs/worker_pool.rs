use std::sync::Arc;

pub struct WorkerPool {
    pool: Arc<rayon::ThreadPool>,
    worker_count: usize,
}

impl WorkerPool {
    pub fn new() -> Self {
        let count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .max(2);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(count)
            .thread_name(|idx| format!("engene-worker-{}", idx))
            .build()
            .expect("failed to create worker pool");
        Self {
            pool: Arc::new(pool),
            worker_count: count,
        }
    }

    pub fn worker_count(&self) -> usize { self.worker_count }

    pub fn execute<F, R>(&self, func: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        self.pool.install(func)
    }

    pub fn par_join<A, B, RA, RB>(&self, a: A, b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.pool.install(|| rayon::join(a, b))
    }

    pub fn pool(&self) -> &rayon::ThreadPool { &self.pool }
}

impl Default for WorkerPool {
    fn default() -> Self { Self::new() }
}
