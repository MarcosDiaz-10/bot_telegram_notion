use crate::{handlers::manager::manager_receive_message, state::SharedState};

use axum::{
    Router,
    routing::{get, post},
};

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/manager", post(manager_receive_message))
        .with_state(state)
}
