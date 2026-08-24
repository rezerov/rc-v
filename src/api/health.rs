use axum::{Json, Router, routing::get};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(Json("Ok")))
}
