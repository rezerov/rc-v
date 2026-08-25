use axum::Router;

use crate::state::AppState;

mod health;
mod jobs;
mod verify;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/job", jobs::routes())
        .nest("/verify", verify::routes())
        .merge(health::routes())
}
