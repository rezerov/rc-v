use axum::Router;

use crate::state::AppState;

mod health;
mod jobs;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/job", jobs::routes())
        .merge(health::routes())
}
