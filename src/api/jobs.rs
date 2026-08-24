use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
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
) -> Result<Json<Job>, JobError> {
    let job: Job = Job {
        id: new.id,
        name: String::from(new.name),
    };
    state.push(job.clone());
    Ok(Json(job))
}

async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Job>, JobError> {
    Ok(Json(state.find_by_id(id)?))
}
