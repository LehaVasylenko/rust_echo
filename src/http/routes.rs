use axum::{routing::{any, get, post}, Router};

use crate::state::AppState;
use super::handler::{echo, health};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/rust", get(health))
        .route("/rust/echo/{*wildcard}", any(echo))
        .route("/rust/echo", post(echo))
        .with_state(state)
}
