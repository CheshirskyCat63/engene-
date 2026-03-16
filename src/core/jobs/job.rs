use std::any::Any;

pub type JobId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub trait Job: Send {
    fn name(&self) -> &str;
    fn priority(&self) -> JobPriority { JobPriority::Normal }
    fn execute(&mut self) -> Box<dyn Any + Send>;
}

pub struct FnJob {
    name: String,
    priority: JobPriority,
    func: Option<Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>>,
}

impl FnJob {
    pub fn new<F>(name: &str, func: F) -> Self
    where
        F: FnOnce() -> Box<dyn Any + Send> + Send + 'static,
    {
        Self {
            name: name.to_string(),
            priority: JobPriority::Normal,
            func: Some(Box::new(func)),
        }
    }

    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }
}

impl Job for FnJob {
    fn name(&self) -> &str { &self.name }
    fn priority(&self) -> JobPriority { self.priority }
    fn execute(&mut self) -> Box<dyn Any + Send> {
        let func = self.func.take().expect("job already executed");
        func()
    }
}
