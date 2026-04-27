use crate::state::SharedState;
use axum::{Router, routing::get};

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}
