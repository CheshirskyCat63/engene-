use std::sync::mpsc;
use std::thread;

pub enum ServiceMessage {
    Task(Box<dyn FnOnce() + Send>),
    Shutdown,
}

pub struct AsyncServiceHandle {
    sender: mpsc::Sender<ServiceMessage>,
    name: String,
}

impl AsyncServiceHandle {
    pub fn submit<F: FnOnce() + Send + 'static>(&self, task: F) -> bool {
        self.sender.send(ServiceMessage::Task(Box::new(task))).is_ok()
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn shutdown(&self) {
        let _ = self.sender.send(ServiceMessage::Shutdown);
    }
}

pub struct AsyncServices {
    handles: Vec<AsyncServiceHandle>,
}

impl AsyncServices {
    pub fn new() -> Self {
        Self { handles: Vec::new() }
    }

    pub fn spawn_service(&mut self, name: &str) -> &AsyncServiceHandle {
        let (tx, rx) = mpsc::channel::<ServiceMessage>();
        let service_name = name.to_string();
        let thread_name = format!("engene-async-{}", name);

        thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                loop {
                    match rx.recv() {
                        Ok(ServiceMessage::Task(task)) => task(),
                        Ok(ServiceMessage::Shutdown) | Err(_) => break,
                    }
                }
            })
            .expect("failed to spawn async service thread");

        self.handles.push(AsyncServiceHandle {
            sender: tx,
            name: service_name,
        });
        self.handles.last().unwrap()
    }

    pub fn get(&self, name: &str) -> Option<&AsyncServiceHandle> {
        self.handles.iter().find(|h| h.name == name)
    }

    pub fn shutdown_all(&self) {
        for handle in &self.handles {
            handle.shutdown();
        }
    }
}

impl Default for AsyncServices {
    fn default() -> Self { Self::new() }
}

impl Drop for AsyncServices {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}
