use std::sync::Arc;

pub struct JobGraph {
    pool: Arc<rayon::ThreadPool>,
}

impl JobGraph {
    pub fn new() -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus())
            .build()
            .expect("failed to create rayon thread pool");
        Self {
            pool: Arc::new(pool),
        }
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

    pub fn pool(&self) -> &rayon::ThreadPool {
        &self.pool
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .max(2)
}
