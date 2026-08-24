use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};

use crate::{
    error::JobError,
    models::{Job, NewJob},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs).post(create_job))
        .route("/{id}", get(get_job))
}

async fn list_jobs(State(state): State<AppState>) -> Json<Vec<Job>> {
    Json(state.list())
}

async fn create_job(
    State(state): State<AppState>,
    Json(new): Json<NewJob>,
) -> Result<(), JobError> {
    let job: Job = Job {
        id: new.id,
        name: String::from(new.name),
    };
    Ok(state.push(job))
}
