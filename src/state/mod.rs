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

    pub fn push(&self, job: Job) -> Result<(), JobError> {
        let mut jobs = self.jobs.write().map_err(|_| JobError::LockPoisoned)?;

        // Check if job already exists
        if jobs.iter().any(|j| j.id == job.id) {
            return Err(JobError::JobIdAlreadyExists);
        }

        jobs.push(job);
        Ok(())
    }

    pub fn find_by_id(&self, id: u64) -> Result<Job, JobError> {
        let jobs = self.jobs.read().map_err(|_| JobError::LockPoisoned)?;

        jobs.iter()
            .find(|job| job.id == id)
            .cloned()
            .ok_or(JobError::NotFound(id))
    }
}
