use crate::Job;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct AppState {
    jobs: Arc<RwLock<Vec<Job>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<Job> {
        self.jobs.read().unwrap().clone()
    }

    pub fn push(&self, job: Job) {
        self.jobs.write().unwrap().push(job);
    }
}
