use std::sync::Arc;

use axum::{Json, Router, routing::get};
use serde::Serialize;
use tokio::sync::Mutex;

mod api;
mod queue;
mod state;

#[derive(Serialize)]
pub struct Job {
    id: u64,
}

type JobQueue = Arc<Mutex<Vec<Job>>>;
let job_queue: JobQueue = Arc::new(Mutex::new(Vec::new()));

#[tokio::main]
async fn main() {
    println!("Job Queue Starting");


    let app = Router::new()
        .route("/", get(hello))
        .route("/jobs", get(list_active_jobs));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> String {
    String::from("hello")
}

async fn list_active_jobs() -> Json<Vec<Job>> {
    Job
}
