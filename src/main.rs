use axum::{Json, Router, extract::State, routing::get};

use crate::{error::JobError, models::Job, state::AppState};

mod api;
mod error;
mod models;
mod state;

#[tokio::main]
async fn main() {
    println!("Job Queue Starting");

    let app = Router::new()
        .route("/", get(hello))
        .route("/jobs", get(list_active_jobs))
        .route("/add_job", get(push_job))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> String {
    String::from("hello")
}

async fn list_active_jobs(State(state): State<AppState>) -> Json<Vec<Job>> {
    Json(state.list())
}

async fn push_job(State(state): State<AppState>) -> Result<(), JobError> {
    Ok(state.push(Job {
        id: 1,
        name: String::from("helo"),
    }))
}
