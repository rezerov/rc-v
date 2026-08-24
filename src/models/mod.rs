use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct NewJob {
    pub id: u64,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: u64,
    pub name: String,
}
