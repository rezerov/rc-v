use axum::Router;

use crate::state::AppState;

mod blocks;

pub fn routes() -> Router<AppState> {
    Router::new().merge(blocks::routes())
}
