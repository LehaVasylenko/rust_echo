use axum::{middleware, routing::{any, get, post}, Router};
use axum::extract::DefaultBodyLimit;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::http::open_api::ApiDoc;
use crate::log::log_request;
use crate::state::AppState;
use super::handler::{echo, health};
use super::ascii::{ascii_handler};

pub fn router(state: AppState) -> Router {
    let limit = 20 * 1024 * 1024;
    let spec = ApiDoc::openapi();
    Router::new()
        .route("/rust", get(health))
        .route("/rust/echo/{*wildcard}", any(echo)).layer(DefaultBodyLimit::max(limit))
        .route("/rust/echo", post(echo)).layer(DefaultBodyLimit::max(limit))
        .route("/rust/ascii", post(ascii_handler)).layer(DefaultBodyLimit::max(limit))
        .merge(SwaggerUi::new("/rust/swagger-ui")
                .url("/rust/api-docs/openapi.json", spec))
        .with_state(state).layer(middleware::from_fn(log_request))
}
