use crate::Job;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct AppState {
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }

    pub fn push(&self, job: Job) {
        self.jobs.lock().unwrap().push(job);
    }
}
