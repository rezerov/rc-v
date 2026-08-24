use crate::{error::JobError, models::Job};
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

    pub fn find_by_id(&self, id: u64) -> Result<Job, JobError> {
        let jobs = self.jobs.read().map_err(|_| JobError::LockPoisoned)?;

        jobs.iter()
            .find(|job| job.id == id)
            .cloned()
            .ok_or(JobError::NotFound(id))
    }
}
